# Sand Dropper

### Project description
This is a small visualizer for a solution to the [Advent of Code 2022 Day 14](https://adventofcode.com/2022/day/14) problem about filling a cave with dropping sand.
I was playing around with ggez for Rust game development and wanted to experiment a bit with rendering many objects.
By my head math the default settings of this project with the provided input map will drop ~20-30,000 grains of sand, and on my computer do it all at a happy 60 FPS :)

![Dropping!}(/dropping.webm)
The dropper in action!

### Writing custom input
Any 'cave' can be input by following the Advent of Code specifications for input, but some constants need to be changed in the code. 
I was too lazy to figure automatically parse the appropriate width and height of the rendered window based on the cave itself, and the cave 'floor' is drawn at a fixed height.
But these changes should be easily configurable, at least!

![The end of a sand drop](/Fully_dropped.png)
This is what the provide input map looks like when completed
