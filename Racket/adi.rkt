#lang racket/base
(require "fixpoint.rkt")
(require racket/match
         racket/set)


;; ============================================================
;; Interpreter (tiered ANF with 0CFA)
;; ============================================================
;;
;; This is an abstract definitional interpreter for ANF (A-Normal Form).
;; It performs 0CFA: a monovariant control-flow analysis where all
;; bindings to the same variable share one abstract address.
;;
;; The interpreter follows the concrete semantics closely:
;;   - Variables are looked up in an abstract store
;;   - Lambdas become closures (but without environments in 0CFA)
;;   - Application nondeterministically applies all possible closures
;;   - Memoization handles recursive calls and ensures termination
;;
;; ANF Syntax (tiered):
;;   aexp ::= x | lit | (λ (x) exp)              ; atomic
;;   cexp ::= (app aexp aexp) | aexp             ; complex
;;   exp  ::= (let ([x cexp]) exp)               ; general
;;          | (if aexp exp exp)
;;          | cexp

;; Abstract store: maps variables to sets of abstract values
;; Implemented as a memoized function — calling (σ x) registers
;; demand for x's value; bind! delivers values to that demand.
(define σ (memo 'σ (λ (x) fail)))

;; bind!: deliver a value to a variable's address in the store
;; Returns empty list (no values to pass on, just a side effect)
(define (((bind! x v) κ) s)
  ((κ '()) (((deliver (cons 'σ (list x))) (list v)) s)))

;; aeval: evaluate atomic expressions
;;   - Variables: look up in abstract store (registers demand)
;;   - Lambdas: return a closure (just the syntax in 0CFA)
;;   - Literals: return as-is
(define (aeval a)
  (match a
    [(? symbol?)   (σ a)]
    [`(λ (,x) ,b)  (return `(clo ,a))]
    [_             (return a)]))

;; ceval: evaluate complex expressions (memoized)
;;   - Application: evaluate function and argument, apply closure
;;   - Otherwise: fall through to aeval
(define ceval
  (memo 'ceval
    (λ (c)
      (match c
        [`(app ,f ,a)
         (>>= (aeval f) (λ (fv)
           (>>= (aeval a) (λ (av)
             (apply-clo fv av)))))]
        [_ (aeval c)]))))

;; eval: evaluate general expressions (memoized)
;;   - let: evaluate RHS, bind result, evaluate body
;;   - if: evaluate condition, take appropriate branch(es)
;;   - Otherwise: fall through to ceval
(define eval
  (memo 'eval
    (λ (e)
      (match e
        [`(let ([,x ,r]) ,b)  (>>= (ceval r) (λ (v)
                               (>>= (bind! x v) (λ ()
                                 (eval b)))))]
        [`(if ,c ,t ,f)       (>>= (aeval c) (λ (cv)
                               (match cv
                                 [#f (eval f)]
                                 [_  (eval t)])))]
        [_ (ceval e)]))))

;; apply-clo: apply a closure to an argument
;;   - Bind the argument to the parameter
;;   - Evaluate the body
;;   - Non-closures fail (no results)
(define (apply-clo fv av)
  (match fv
    [`(clo (λ (,x) ,b))
     (>>= (bind! x av) (λ () (eval b)))]
    [_ fail]))

;; ============================================================
;; Examples
;; ============================================================

(define (show-store tbl)
  (for/hash ([(k v) (in-hash tbl)]
             #:when (and (pair? k) (eq? 'σ (car k))))
    (values (cadr k) (car v))))

(displayln "--- let binding ---")
(displayln (show-store (run (eval '(let ([x 1]) x)))))

(displayln "\n--- identity function ---")
(displayln (show-store (run (eval '(let ([f (λ (y) y)])
                                     (app f 42))))))

(displayln "\n--- multiple values ---")
(displayln (show-store (run (eval '(let ([f (λ (x) x)])
                                     (let ([a (app f 1)])
                                       (let ([b (app f 2)])
                                         a)))))))

(displayln "\n--- conditional (true) ---")
(displayln (show-store (run (eval '(if #t (let ([x 1]) x) (let ([x 2]) x))))))

(displayln "\n--- conditional (unknown) ---")
(displayln (show-store (run (eval '(let ([b (λ (x) x)])
                                     (let ([c (app b #t)])
                                       (let ([d (app b #f)])
                                         (if c (let ([r 1]) r) (let ([r 2]) r)))))))))

(displayln "\n--- higher-order ---")
(displayln (show-store (run (eval '(let ([apply (λ (f) (app f 99))])
                                     (let ([g (λ (z) z)])
                                       (app apply g)))))))

(displayln "\n--- self-application ---")
(displayln (show-store (run (eval '(let ([ω (λ (x) (app x x))])
                                     (app ω ω))))))
