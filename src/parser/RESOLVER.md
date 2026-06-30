## General Approach

From the parsed output, we have a list of definitions (noted if they are global or local), as well as their ids. We traverse the AST constructed by the parser, and whenever we encounter a Text block, we must match on the text.

## Resolution Structure

We start with internal representation of a Text block: 
    
    Node::Text {id: Vec<u32>, text: String }

Now, we will create a new enum type:
    
    enum Text = PlainText(String) | Ref { term: String (could disclude this, but will have to do lookup for each term?), ids: Vec<Vec<u32>>}

NOTE: for storage purposes, we may want to create a separate, flat ID schema for definitions as to not require Vec<Vec<u32>> to house a list of IDs 

We will also need to appropriately modify our definition of Node::Text to fit:

    Node::Text { id: Vec<u32>, text: Vec<Text> }

ex. Suppose we start with the following Text block:

    {
        id: [0, 1, 3],
        text: "I think that machine learning is useful"
    }

We transform this into representation:

    {
        id: [0, 1, 3],
        text: [
            PlainText("I think that "),
            Ref { term: "machine learning", ids: [[0, 1], [0, 1, 5]] },
            PlainText(" is useful")
        ]
    }


## String Matching

We will use the Aho-Corasick crate which implements trie (prefix tree) construction and matching, in order to construct a prefix tree of all definitions.

## Locality Resolution

We need to id blocks/use a data structure st. we can, for any given block, tell if a local definition applies. So, maybe we have id be a multi-layer structure.

For example, if we have the following layout:

                    block
                /           \
        node1                   node2
       /     \                 /     \
child1      child2      child3      child4

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

