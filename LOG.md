# 2025-04-27

Implemented iterator and added benchmarks for my splay implementation.

I don't know why it is ~2x slower than splay implementation from a random
crate that I found. Wanted to use Instruments.app but of course it's broken
and I don't want to deal with it right now.

A silly mistake I made was writing bench closure which cloned the vector
on each iteration, which was... unfortunate. Learned the difference between
`iter()` and `into_iter()` though!

Still, the sorting benchmark is running out of memory on my system and
I don't know why.

# 2025-04-26

It was a good exercise to implement splay tree! I got through some common
painpoints of rust (like not being able to get a mutable borrow to a part
of a structure while holding mutable borrow for the whole structure) but
I managed to avoid most of the issues by referring to nodes by array index.

I like my code for splay tree, it's actually kinda short and nice? I didn't
see this kind of implementation before.

# 2025-04-23

Learning cargo and basics of Rust. I think I'll write a very simple in memory
key-value store as a warm up. I'll start with the data structure part and do
networking later. Seems like a more sensible starting point for learning a
new language.

Memtable has to be able to do several operations:

- Set(key, value): we can squash multiple updates into one, keep the last one.
- Delete(key): we can't just remove the key from memtable, the key might be
  present in lower levels, so I need to propagate the tombstone.

For each key, I need to hold at most one operation: either a new value for that
key, or a tombstone. I can store Vec<u8> option which should actually just represent
none as nullptr.

Second question, what data structure should I use? The classic choice is to use
some sort of BST but I'm not a fan of all of the pointer chasing. Let's stick to
my self-imposed goals and choose something which I can implement in small amount
of lines of code.

A possible solution is to just keep a vector of operations and sort it when flushing,
why would that be bad? If I get a lot of operations on the same key, I'll store
values which I could've deallocated already. What's better?

The vector solution requires sorting at the end, which doesn't help much. I'll do
BST, I don't need delete operation on it anyway which should keep the implementation
quite simple (fingers crossed).

I also have a second idea: I could keep everything in a compressed trie. There's just
one comparison function (lexicographic on sequence of bytes) so I could swing it.
It's an interesting design space, how wide should my nodes be, when to switch from
linear scan to 256-wide array etc. Fun!
