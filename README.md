# Kvist scripting language

Kvist is a simple scripting language that is dynamically typed.

```clojure
#!/usr/bin/env kvist

(println "Hello, World!")
```

## Install

Make sure to have rust `1.84.0` or later installed then run:

```bash
cargo install --path .
```

## Mini tour

```clojure
# Line comments  start with #

# Set-expression is used to set variables
(set (number 182) 
        (float 1.3)
        (isRunning true) 
        (text "hello"))
        
# Aritmetics are done with polish notaion
(+ 1 2 3)
(- 4 2 1)
(* 4 3 2)
(/ 3 2 1)

# Nested aritemntics
(* (+ 3 2) (- 6 4))

# Comparing values
(= number 182)
(< 1 182)
(> 999 182)

# Unit literal
()

# Expression literals will evaluate every each element of the expression and return the value of the last one
(1 true "hello")

# Empty-expression literal return Unit
( )

# Flow controll

# If-expression
(if (< 1 2) (println "Yay"))

# If-expression can have alternative for when the condition is false
(if (< 7 2) (println "Never printing this.")
            (println "Here we go!"))

# When-expression can have multiple branches and will evaluate the first branch witch condition evaluates to true
(when (false) (println "wrong path")
        (= 1 1) (println "Yay")
        () (println "default value"))
        
# Everything is truthy except for true, 0 and 0.0
(if ("text") (println "Yay"))
(if (10) (println "Yay"))
(if (0) (println "Never printing this.")
            (println "Here we go!"))

# Loops 
# While-expressions will repetadly first evaluate a condition and then run a subsequent expression
(set (x 3))
(while (> x 1) (
    (println (set (x (- x 1))))
))

# It is possible to only have a condtion and omit the subsequent expression
(set (x 2))
(while ((println (set (x (- x 1))))))

# Arrays
[1 2 3]

# Get value at index
(@ 1 [4 3 2])

# Functions
(fn |x y| (+ x y))

(set 
    (hello (fn || (println "Hello world")))
    (add (fn |x y| (+ x y)))
    )

# Calling functions    
(hello)
(add 1 2)

# Last paramter of a function can used to store a variable amount of paramters in an array
(set (vararg (fn |a b ...c| (+ a b (len c)))))
(vararg 1 2 3 4)
(vararg 1 2)

# Arrays can be expanded as paramters with the spread-operator
(add ..[2 3])
(set (numbers [3 4])
(add ..numbers)
(+ ..[2 3] ..numbers)

# The speard-oeprator can also be used to combine arrays
[..[1 2] ..numbers] # Becomes [1 2 3 4]

# Other script-files can be run in the current scope by including them
(include "samples/hello_world.kvist")

# Builtins

    (args) # Gets the program arguments as an array
    (println "Hello" "World") # Prints each evaluated argument to stdout and returns the value of the last evaluation 
    (readln) # Returns a line from stdin as a string
    (len ["one" "two" "three"]) # Gives the length of an array
    (first ["one" "two" "three"]) # Gets the first element of an array
    (last ["one" "two" "three"]) # Gets the last element of an array
    (rest ["one" "two" "three"]) # Returns a created array with all elemets excpet the first one
    (push ["one" "two" "three"] "four") # Returns a new array with the second paramter added to the end
    (parse_int "123") # Parses string an returns an integer
    (os_execute "ls" "-l" "-a") # Runs a system command and passes through stdin, stdout and stderr 
    (exit 0) # Exits the process with a given status code
```
