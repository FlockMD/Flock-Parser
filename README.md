## Flock Markdown Language Parser

### Parser Grammar:


```
BLOCK = ( TEXT | DEF | IMG | LATEX | SECTION )*

TEXT = TEXT* | Sigma* //“normal string of text”

SECTION = @section(BLOCK?) { ( BLOCK | SECTION )* }

DEF = @def(IDENT) { BLOCK }

LOCAL = @local(IDENT) { BLOCK } // may make part of DEF rule? Treated same as def but only remains in scope of BLOCK of local def

IMG = @img(PATH)

PATH = (/Sigma*)* // "path/formatted/string”

LATEX = @latex(latex math expression here’)

IDENT = TEXT (for now)
```

-------------------------------------------------------

## Parsing Strategy:


## Definition Mapping Strategy:

For the purpose of clarity, we'll refer to any string of text with a definition as a 'term'. This 'term' could be composed of multiple words (such as 'deep neural network'), as long as it has a definition.

During the parsing step, every time a new term is encountered, we'll add it to a hash map that maps terms to their definitions.

After parsing, we'll move forward with the following steps:

1. Preprocessing
First, we will construct a prefix tree from the list of terms defined to allow for quick text matching 

2. Definition mapping:
For each text block, we will scan all words (can be made easier by parsing each disconnected word into a separate node in the tree (a TEXT block is a list of WORD blocks)). For each word, we will use our prefix tree to find its definition (each 'end' node in the tree actually directly stores a pointer to a slice containing the definition

 --> NOTE: we now need to figure out an easy way to encode this processing into JSON format to export to the frontend (maybe its a table/map of word_number_in_text_block -> definition_number_in_def_list

-------------------------------------------------------

Potential Problems:

1. How to deal with multiple definitions for the same term. 
Either overwrite to take most recent, or store all (better option).

2. How to deal with plural version of defined word
For example, say I define the term eigenvalue. If I use the term 'eigenvalues', easy enough to match by appending an 's' onto the end.

Suppose instead, I define a term like 'fly'. Its plural, 'flies', cannot be tracked so easily. Potentially some API out there to get plural versions, could also cache any instance needed to use API to not make repetitive calls for matching in a separate mapping of word -> word's plural version.

-------------------------------------------------------

# Additional Features:

## Custom Rules:
I want users to be able to define custom rules in their markdown of the form @WORD(TEXT) that automatically provides styling rules to that text. They would provide this in a config file which contains form:

```
WORD 
{
    RULE,
    RULE,
    ...
}
```

Where each rule has the form:

```
KEY: VALUE
```

Will need to define list of rule options, such as size, colour, etc. May also provide option to override default styling options
--> might want to change definition rule from @IDENT(BLOCK) to @def(IDENT, BLOCK)
    --> then, I can override styling rules for @def

For example, often when I'm writing an example I use purple text, and prefix with ex.

So, I can create rule

```
ex
{
    color: purple,
    prefix: "ex. "
    ...
}
```

Then, in my note, if I encounter the following:

```
@ex(
    Suppose I have 3 balls and 2 bins.
)
```

It will be formatted in the following way (assume the text becomes purple):

ex. Suppose I have 3 balls and 2 bins.

## Scope-Level Definitions:

Suppose I only want to restrain certain definitions to a specific BLOCK, SECTION, FILE (instead of group of files). I want to be able to create a scope-level definition.

I can do this by assigning each BLOCK a unique ID, then in the JSON file of defs, I include that ID (or reserve ID 0 for global), that way when mapping i can look through only the place I need.

// NOTE: add additional @local(IDENT, BLOCK) to parser for local definitions that automatically associates with block it is nested in (or file if its at the top level, must also then assign a unique ID to each file)

--> This can allow me to also add additional info in only certain contexts. For example, suppose I am taking notes in my ML course, and I have global def:
    // note below, since ',' used to delimit args here, may need to rethink if each arg needs to be nested in a "", or use a better/less frequent delimiter than ','

```
@def(Forward Propogation) { The process of pushing values through a neural network through layers and get a final output }
```

Then, suppose I'm taking notes in an RNN section, I can add context about how previous hidden states are used recurrently in later outputs and hidden states, ex.

```
@section(RNN) {
    ...

    @def(Forward Propogation) { Uses previous hidden states that contribute to the values of later ones (as well as output(s)) }
    ...
}
```

Then, whenever referencing this definition later inside the RNN section, the popup will display something like:

```
 Forward Propogation: The process of pushing values through a neural network through layers and get a final output
    |
    -----> Uses previous hidden states that contribute to the values of later ones (as well as output(s))
```

-------------------------------------------------------

## Locality Resolution

We need to id blocks/use a data structure st. we can, for any given block, tell if a local definition applies. So, maybe we have id be a multi-layer structure.

For example, if we have the following layout:
```

                    block
                /           \
        node1                   node2
       /     \                 /     \
child1      child2      child3      child4
```

We assign the following ids:
block -> 0
node1 -> 0/0
node2 -> 0/1
child1 -> 0/0/0
child2 -> 0/0/1
child3 -> 0/1/0
child4 -> 0/1/1

Consider the following example:

We have created a local def in 1/1. If we are currently inside block 1/1/0/2, we match up to 1/1. Then, since we've matched to completion, the local def applies within this block.

