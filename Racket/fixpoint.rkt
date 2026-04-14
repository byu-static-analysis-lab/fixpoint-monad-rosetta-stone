#lang racket/base

(require racket/match
         racket/set)
(provide return >>= each fail memo deliver run)
;; ============================================================
;; Monad (curried, multi-value)
;; ============================================================
;;
;; This is a continuation monad composed with a state monad.
;; The state (s) is a memo table mapping keys to (values . continuations).
;;
;; The monad enables:
;;   - Nondeterminism: exploring multiple abstract values
;;   - Memoization: caching results and detecting fixed points
;;   - Demand-driven iteration: when new values appear, they are
;;     delivered to all waiting continuations
;;
;; Type: M a = (list → s → s) → s → s
;;   - Takes a continuation κ that receives a list of values
;;   - Returns a state transformer (s → s)

;; M = (list → s → s) → s → s

;; return: inject values into the monad
(define ((return . vs) κ) (κ vs))

;; >>=: sequence computations, spreading the value list to f via apply
(define ((>>= c f) κ)
  (c (λ (vs) ((apply f vs) κ))))

;; each: nondeterministic choice — run all computations, threading state
(define (((each . cs) κ) s)
  (foldl (λ (c s) ((c κ) s)) s cs))

;; fail: no results (each with zero computations)
(define fail (each))

;; memo: memoize a function with demand-driven fixed point computation
;;   - First call: register continuation, compute, deliver results
;;   - Later calls: register continuation, deliver cached results
;;   - When new values arrive via deliver, all waiting continuations receive them
(define ((memo tag f) . args)
  (let ([key (cons tag args)])
    (λ (κ)
      (λ (s)
        (match (hash-ref s key #f)
          ;; First time: init entry, add κ to waiters, compute
          [#f
           (let ([s′ (hash-set s key (cons (set) (list κ)))])
             (((apply f args) (deliver key)) s′))]
          ;; Seen before: add κ to waiters, deliver cached values
          [(cons vals conts)
           (let ([s′ (hash-set s key (cons vals (cons κ conts)))])
             (for/fold ([s s′]) ([vs (in-set vals)])
               ((κ vs) s)))])))))

;; deliver: propagate a new value to all waiting continuations
;;   - Once given a key, (deliver key) is itself a continuation
;;   - If value already seen, do nothing (fixed point for this value)
;;   - Otherwise, add to cache and notify all waiters
(define (((deliver key) vs) s)
  (match-let ([(cons vals conts) (hash-ref s key (cons (set) '()))])
    (if (set-member? vals vs)
        s
        (let ([s′ (hash-set s key (cons (set-add vals vs) conts))])
          (for/fold ([s s′]) ([κ (in-list conts)])
            ((κ vs) s))))))

;; run: execute a computation, return final memo table
(define (run c)
  ((c (λ _ (λ (s) s))) (hash)))
