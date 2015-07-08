Game Engine
===========

This is a toy "game engine" written in Rust. It is not a proper game engine, as currently the game and the engine are coupled together, so a game would be implemented by cloning the project and implementing the game in the main loop. This coupling is not due to any technical limitations, however; decoupling is just something I haven't gotten around to yet.

Features
--------

- Supports OpenGL
- The engine supports rendering 3D objects by either manually specifying vertices or by supplying an `obj` file containing the vertices. The objects must implement the `GameObject` trait, which allows the engine to get the necessary information to draw and update the object.
- Text rendering is supported.
- Texture and glyph caching
- A perspective camera is provided
- A 2D GUI shader and a 3D perspective shader are provided. An unlit fragment shader is also provided. Both colors and textures can be used with the unlit fragment shader.

Example
-------

Running the engine in it's current state will run an example. The example consists of a grid, a rotating cube, some text, and the ability to translate (left click), rotate (right click), and zoom  (scroll wheel) the camera:

![demo](demo.gif)

To add your own objects, implement `GameObject` on them using `Grid` and `Cube` as an example.

Future Improvements
-------------------

While I'm not sure if development will continue on this project, if it did these are some features I would add:

- Lighting objects/shaders
- Diffuse and specular shaders
- An orthographic camera
- Easier ways to transform (translate, rotate and scale) objects
- Quaternions
- Frustrum culling
- Render children objects relative to parent instead of in absolute coordinates
