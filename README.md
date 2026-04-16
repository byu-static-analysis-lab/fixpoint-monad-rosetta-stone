# Fixpoint Monad Rosetta Stone
Here is our fixpoint monad in a few languages

Blog post is below

# A Small yet powerful tool for your belt

What do recursive-decent parsing, datalog and program analysis all have in common? They all used fixed points to efficiently solve their respective problems. Let's see how this is the case.

We have a small library that we use in our lab that enables us to do our work efficiently. This library can be used for all sorts of things from parsing, abstract interpretation, implementing Datalog, performing type inference. This article will first go over the basics of what fixed points are, then we will provide a Datalog example to gain an intuition on both how a fixpoint solver works and how our library functions. Finally, we will go over how you can use our library to build a parser. At the end, we will link our GitHub repository which will contain the implementation in a few languages as well as some examples.

## What is a fixpoint?
A fixpoint (or fixed point) of a function is a value that maps to itself. For example for the function `f(x) = x^2`, the fixpoint of this function is `0` and `1` since `x = x^2` for those situations. You can think of a fixpoint as a stable solution for a problem where input doesn't change output. In a Datalog interpreter, the inputs and output of an interpreter are a set of facts.
Take the following example that models a parent-child relationship.
```datalog
parent(tom, bob).
parent(bob, ann).
parent(bob, pat).

ancestor(X, Y) :- parent(X, Y).
ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
```
The set of initial facts are that Bob is a child of Tom, Ann is a child of Bob, and Pat is a child of Bob.
### What is a fixpoint solver
A fixpoint solver's job is to find this stable solution for a given equation because many fixpoints of functions are not trivial to produce. The solver's job is to apply a function on a given input until the sequence of output converges. In other words, the limit of this iteration is the fixpoint of the function for the given input.

For our Datalog example we will want to find this relationship `ancestor(tom, ann)?` we want to know if there is a transitive relationship between Tom and Ann.

### Where fixpoint solvers pop up in computer science
They are primarily used in static analysis techniques in data flow analysis where we want to know what values a variable may hold, it powers type inference since the compiler needs to figure out what type of a value is so that the code can be compiled correctly, and it is used in abstract interpretation where we want to prove certain program properties. In our lab we have come up with a novel use of a fixpoint solver for parsing. We well show this off in a later section.

## A high level understanding of our library
Before we go into examples or how the library itself. It is best to gain an intuition for how our library works.
Earlier it was mentioned that a fixpoint solver is iterative. So we know that we have some kind of looping structure. For input we will have some state. This state is a mapping from an identifier to a set of possible values. In general, we know when a solver is done because either we don't learn any new mappings or we don't learn any new possible values. Then it is a matter of seeing if the desired value is in the right mapping.

### The Naive Approach
The naive approach is to loop over our mappings and see if we can make progress. New facts emerge from evaluation and then we keep on iterating until no more progress can be made.

#### Datalog Example
Recall our example earlier
```datalog
parent(tom, bob).
parent(bob, ann).
parent(bob, pat).

ancestor(X, Y) :- parent(X, Y).
ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
```
We will run through this example with the naive approach.
First, we set up our initial state as follows
```
{
	parent(tom, bob) ↦ { true }
	parent(bob, ann) ↦ { true }
	parent(bob, pat) ↦ { true }
}
```
We have an initial mapping from each starting fact to a set of values that are known.

We now want to know if this is true `ancestor(tom, ann)?`
We load this into our facts.
Our facts are represented as a pair of values. The expression in question and whether it is true or not.
```
{
	parent(tom, bob), true
	parent(bob, ann), true
	parent(bob, pat), true
}
```

This is isomorphic to this mapping with empty set meaning not known, and the singleton set, known. This will be more useful when we introduce our library, so keep this in mind.
```
{
	parent(tom, bob) ↦ { true }
	parent(bob, ann) ↦ { true }
	parent(bob, pat) ↦ { true }
}
```

We leave the set empty because we don't know what the value might be.
So we start iterating.
For the first step we need to first apply these rules
```
ancestor(X, Y) :- parent(X, Y).
ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
```
When we evaluate that, we get these result
```
{
	parent(tom, bob), true
	parent(bob, ann), true
	parent(bob, pat), true
	ancestor(tom, bob), true
	ancestor(bob, ann), true
	ancestor(bob, pat), true
}
```
We unfortunately don't learn anything useful yet
Luckily we now have the knowledge to try to apply the second rule for ancestor.
However, during this process, we have to run through the first rule again, learning nothing new but wasting time.
So now we learn this
```
{
	parent(tom, bob), true
	parent(bob, ann), true
	parent(bob, pat), true
	ancestor(tom, bob), true
	ancestor(bob, ann), true
	ancestor(bob, pat), true
	ancestor(tom, ann), true
	ancestor(tom, pat), true
}
```
We have now found the result we are looking for. However, we need to iterate again to see if we learn any new facts.
We iterate again and run through the rules again, wasting time to learn nothing.
```
{
	parent(tom, bob), true
	parent(bob, ann), true
	parent(bob, pat), true
	ancestor(tom, bob), true
	ancestor(bob, ann), true
	ancestor(bob, pat), true
	ancestor(tom, ann), true
	ancestor(tom, pat), true
}
```
Since we didn't change from our last iteration, we know know that we have found the fixed point of this Datalog program.

#### Downsides to the naive approach
The issue with the naive approach is that we end up doing the same amount of work multiple times with each pass of the fixpoint solver. This gives us a rough time complexity of O(n^2) but is really O(n \* m) with 'n' being the rate of convergence (amount of iterations) and 'm' being the cost of the fixpoint function. This isn't great. Luckily, our approach exploits some useful observations about how the solver behaves.
While we can speed up the doing extra work by memoizing the work already done, this still doesn't handle the fact that we are still doing some work that we have already solved.

### Our Approach
There is a useful observation about how data flows through a fixpoint solver. You end up getting a dependency graph. We exploit this by storing callbacks in a state's value in addition to a list of possible values. The callbacks are invoked whenever a new possible value is learned of. This means that whenever we make progress, we only notify whoever needs to know about the new values. This improves performance tremendously. When used in conjunction with memoization, we ensure only meaningful work is accomplished.

#### Datalog Example
Recall our example earlier
```datalog
parent(tom, bob).
parent(bob, ann).
parent(bob, pat).

ancestor(X, Y) :- parent(X, Y).
ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
```
We will run through this example with the our approach.
First, we set up our initial state as follows.
We are using a map instead, using the expression as a key and a tuple that contains the set of possible answers, and a list of dependents.
```
{
	parent(tom, bob) ↦ { true }, []
	parent(bob, ann) ↦ { true }, []
	parent(bob, pat) ↦ { true }, []
}
```
We have an initial mapping from each starting fact to a set of values that are known.

We now want to know if this is true `ancestor(tom, ann)?`
We load this into our facts.
```
{
	parent(tom, bob) ↦ { true }, []
	parent(bob, ann) ↦ { true }, []
	parent(bob, pat) ↦ { true }, []
}
```
Nothing that different so far but lets see how iteration works.

Our approach both memoizes and tracks demand on the fly. This leads us to be non-deterministic.
So lets say that we evaluated the second rule of ancestor first. We will end up with this result.
```
{
	parent(tom, bob) ↦ { true }
	parent(bob, ann) ↦ { true }
	parent(bob, pat) ↦ { true }
	ancestor(tom, ann) ↦ { } [Demand(parent(X, ann)) and Demand(parent(tom, Y))]
	ancestor(tom, pat) ↦ { } [Demand(parent(X, pat)) and Demand(parent(tom, Y))]
}
```
Since we already know of the facts in demand for `ancestor(tom, ann)` and `ancestor(tom, pat)`, we evaluate it immediately and get this result in our iteration.
```
{
	parent(tom, bob) ↦ { true }, []
	parent(bob, ann) ↦ { true }, []
	parent(bob, pat) ↦ { true }, []
	ancestor(tom, ann) ↦ { true }, [Demand(parent(X, ann)) and Demand(parent(tom, Y))]
	ancestor(tom, pat) ↦ { true }, [Demand(parent(X, pat)) and Demand(parent(tom, Y))]
}
```
And in two iterations we have found what we are looking for.
We do now have to find the other facts and those simply load in.
```
{
	parent(tom, bob) ↦ { true }, []
	parent(bob, ann) ↦ { true }, []
	parent(bob, pat) ↦ { true }, []
	ancestor(tom, ann) ↦ { true }, [Demand(parent(X, ann)) and Demand(parent(tom, Y))]
	ancestor(tom, pat) ↦ { true }, [Demand(parent(X, pat)) and Demand(parent(tom, Y))]
	ancestor(tom, bob) ↦ { true }, []
	ancestor(bob, ann) ↦ { true }, []
	ancestor(bob, pat) ↦ { true }, []
}
```
We iterate one more time and learn nothing new.
It should note that memoization will cut down on the work needed to be done every step, that is just not reflected with the simple example that we had.


## How our library works
Our library uses a monad to handle modularity. We will not discuss what a monad is in this blogpost. This just means that as long as you have higher order functions in your language, you can implement our algorithm.
Our memoizing fixpoint monad is composed with a state monad. We use continuations as a callback mechanism to handle delivery of new facts when demanded.

## Fixed point parsing
A novel use case our lab has come up with is viewing parsing as a fixed point problem. Before going into it, we will need some background knowledge of parsing to demonstrate the motive.

### Motivation
Most students learn about parsing via recursive decent, where functions represent rules in a grammar. This approach is used because of its one-to-one mapping between the code and the grammar. This expressive power makes implementation easy since you simply have to follow the grammar rules and then you are done. We can then extend this with parser combinators to build reusable parsers that compose extremely well and make writing parsers a breeze.

The Achilles' heel for recursive decent is that it cannot handle what is called left recursion. Left recursion is when a grammar rule expands on the left side of either itself or another rule indirectly. This isn't too bad as we can rewrite the grammar to accommodate this. However, the downside is that now you are changing your grammar and therefore implementation. Or just the implementation and then you lose the one-to-one mapping.

Just in case you didn't comprehend left recursion, this is what it looks like in Python.
```python
def parse_minus(input, index):
	left, index = parse_minus(input, index)
	_, index = parse_minus_symbol(input, index)
	right, index = parse_minus(input, index)
	return MinusExpr(left, right), index
```
As you can see, we will infinitely try to call `parse_minus` until we run out of stack space and crash.

### Fixed Point Parsing
When viewing parsing as a fixed point problem, view grammar rules as facts that have results associated with them.
This is powerful because it allows us to have a recursive decent framework that can have left recursion.

#### Example Subtraction
Subtraction in math is left associative, meaning that evaluating the left hand side is the first step before we can proceed.

So in BNF we have this
```
<number> ::= Number ; we are shortcutting the grammar for simplicity
<minus> ::= "-"
<subtract> ::= <subtract> <minus> <number> | <number>
```

In Python, the a recursive decent implementation could look something like this:
```python
def number(input, index):
	"""
	Again, we are waving our hands for this
	"""
	return the_number, new_index
	
def minus(input, index):
	if input[index] == '-':
		return None, index + 1
	else:
		raise UnexpectedInput(index)

def subtract(input, index):
	try:
		left, index = subtract(input, index)
		_, index = minus(input, index)
		right, index = number(input, index)
		return SubtractExpr(left, right), index
	except UnexpectedInput as e:
		number, index = number(input, index)
		return number, index
```
Again we will crash when we try to call subtract on any input

So how do we fix it?

Easy, by tracking demand and memoizing each step.

##### Run through
Lets parse `1 - 1`
For now, lets assume that whitespace is ignored and doesn't factor in. We are just using it for readability.


We start with our facts
```
{
	
}
```
In this case we start with nothing

We will first try to apply `subtract` rule.
We establish demand for another subtract, a minus, and a number or just a number
This leaves us with this state
```
{
	<subtract> ↦ {}, [Demand(subtract, minus, number), Demand(number)]
}
```
We now have a demand for a resulting subtract rule.
Since we can't make progress, we continue the iteration and try to parse a number as the other rule of subtract
```
{
	<subtract> ↦ {}, [Demand(subtract, minus, number), Demand(number)]
	<number> ↦ { ("1", 0) }, []
}
```
We now have something.
Notice how we are now storing a tuple of what we parsed with the index is succeeded at.

Since `<number>` is a rule we have for subtract, we propagate demand through and get this:
```
{
	<subtract> ↦ { ("1", 0) }, [Demand(subtract, minus, number), Demand(number)]
	<number> ↦ { ("1", 0) }, []
}
```
Since we had a demand on subtract from subtract, we pass that value down into subtract and make progress on minus:
```
{
	<subtract> ↦ { ("1", 0) }, [Demand(subtract, minus, number), Demand(number)]
	<number> ↦ { ("1", 0) }, []
	<minus> ↦ { ("-", 1) }, []
}
```
Since we made progress on minus, we can now parse another number
```
{
	<subtract> ↦ { ("1", 0) }, [Demand(subtract, minus, number), Demand(number)]
	<number> ↦ { ("1", 0), ("1", 2) }, []
	<minus> ↦ { ("-", 1) }, []
}
```
Since we found another number, we can now complete the rule for subtract
```
{
	<subtract> ↦ { ("1", 0), ("1 - 1", 2) }, [Demand(subtract, minus, number), Demand(number)]
	<number> ↦ { ("1", 0), ("1", 2) }, []
	<minus> ↦ { ("-", 1) }, []
}
```
We know that we are done when one of the tuples contains the length of the input string since we can't make any progress after that point.

## The end
Hopefully you learned something useful that you can apply to your programs.