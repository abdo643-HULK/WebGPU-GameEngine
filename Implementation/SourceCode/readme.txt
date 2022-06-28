http://graphics.stanford.edu/data/3Dscanrep/
http://graphics.stanford.edu/papers/zipper/zipper.pdf
https://on-demand.gputechconf.com/gtc/2016/events/vulkanday/Migrating_from_OpenGL_to_Vulkan.pdf
https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial
https://www.youtube.com/watch?time_continue=402&v=OIfqWD5NlNc&feature=emb_title

https://alain.xyz/blog/comparison-of-modern-graphics-apis


https://github.com/gpuweb/gpuweb/pull/1217 // Fence
https://github.com/gpuweb/gpuweb/pull/1169
https://github.com/gpuweb/gpuweb/issues/1073
https://github.com/gpuweb/gpuweb/pull/1306


# Lerning
## TU Wien Vulkan Series
https://www.youtube.com/watch?v=tLwbj9qys18&list=PLmIqTlJ6KsE1Jx5HV4sd2jOe3V1KMHHgn&index=1
https://www.cg.tuwien.ac.at/courses/EinfCG/slides/VulkanLectureSeries/ECG2021_VK01_Essentials.pdf

## Oregon State Uni Vulkan Series
https://web.engr.oregonstate.edu/~mjb/vulkan/
https://web.engr.oregonstate.edu/~mjb/vulkan/Handouts/ABRIDGED.2pp.pdf
https://web.engr.oregonstate.edu/~mjb/vulkan/Handouts/FULL.1pp.pdf

https://www.construct.net/en/blogs/ashleys-blog-2/porting-webgl-shaders-webgpu-1576

https://vkguide.dev/docs/chapter-5/drawing_images/

stride Attribute replacement: https://github.com/gpuweb/gpuweb/issues/2493
coordinate System:https://github.com/gpuweb/gpuweb/issues/416


https://www.sie.com/content/dam/corporate/jp/guideline/PS4_Web_Content-Guidelines_e.pdf
https://groups.google.com/g/webgl-dev-list/c/Kd4UaVmki-g // webgl2 xbox series

http://www.cross-code.com/en/home

https://link.springer.com/article/10.1007/s00371-021-02152-z


https://vkguide.dev/docs/extra-chapter/asset_system/
https://blender.community/c/rightclickselect/b9fbbc/?sorting=hot
https://developer.nvidia.com/vulkan-turing
https://silverweed.github.io/assets/docs/distributed_rendering_in_vulkan.pdf


// asset streaming
https://docs.unrealengine.com/5.0/en-US/level-streaming-in-unreal-engine/
https://docs.unrealengine.com/5.0/en-US/texture-streaming-in-unreal-engine/

https://docs.unrealengine.com/4.27/en-US/BuildingWorlds/LevelStreaming/
https://docs.unrealengine.com/4.27/en-US/RenderingAndGraphics/Textures/Streaming/

// server streaming
https://docs.unrealengine.com/5.0/en-US/pixel-streaming-in-unreal-engine/
https://medium.com/swlh/webgpu-vs-pixel-streaming-a-view-from-afar-18c7819db2fd

# Info

glb - binary glTF
SPIR-V (Standard Portable Intermediate Representation for Vulkan)

spv file magic number: 0x07230203


Orthographic Camera: Model Viewer
Perspective Camera: Games

hen using orthographic projection, each of the vertex coordinates are directly mapped to clip space without any fancy perspective division (it still does perspective division, but the w component is not manipulated (it stays 1) and thus has no effect). Because the orthographic projection doesn't use perspective projection, objects farther away do not seem smaller, which produces a weird visual output. For this reason the orthographic projection is mainly used for 2D renderings and for some architectural or engineering applications where we'd rather not have vertices distorted by perspective. Applications like Blender that are used for 3D modeling sometimes use orthographic projection for modeling, because it more accurately depicts each object's dimensions. Below you'll see a comparison of both projection methods in Blender