# WebGPU

- NO GLOBAL STATE (OpenGL)

## Concerns Web
- [hidden crypto-mining, password cracking or rainbow tables computations](https://gpuweb.github.io/gpuweb/#security-abuse-of-capabilities)

## Timelines
A computer system with a user agent at the front-end and GPU at the back-end has components working on different timelines in parallel

- Content timeline
- Device timeline
- Queue timeline






# WGSL
- shader is a little program
- 3 entry points/stages (vertex, fragment, compute)
- 2 controllable Pipleines :
    - GPUComputePipeline
    - GPURenderPipeline
- vertex stage must always return `@builtin(position)` either directly or as a property on a struct
- fragment stage must always get an input with `@builtin(position)`
- compute doesn't have a return type
- No manual Memory Duaration
- Address spaces
    - function (function scope)
    - private (module scope)
    - workgroup (module scope)
    - storage (module scope)
    - uniform (module scope)
    - handle (module scope)
- No Recursion
- resources: 
    - shared between all shader invocations
    - 4 types:
        - uniform buffers
        - storage buffers
        - textures
        - samplers
    - must have `@group(N) @binding(M)` attributes
- alised memory views (ownership)
    ```rust
    var x : i32 = 0;

    fn foo() {
        bar(&x, &x); // Both p and q parameters are aliases of x.
    }

    // This function produces a dynamic error because of the aliased
    // memory accesses.
    fn bar(p : ptr<private, i32>, q : ptr<private, i32>) {
        if (x == 0) { // not allowed to use x
            *p = 1;
        } else {
            *q = 2;
        }
    }
    ```
- extensions are possible 
    - proposal to add [RayTracing](https://github.com/gpuweb/gpuweb/issues/535)
    - To enable extension use: enable directive
        ```rust
        // Enable a hypothetical extension for arbitrary precision floating point types.
        enable aribtrary_precision_float;
        enable arbitrary_precision_float; // A redundant enable directive is ok.
        ```
- Workgroup: 
    - Current Issue: 
        ```
        Can we query upper bounds on workgroup size dimensions? Is it independent of the shader, or a property to be queried after creating the shader module?
        ```
    - Example:
        ```rust
        @compute @workgroup_size(8,4,1)
        fn sorter() { }

        @compute @workgroup_size(8u)
        fn reverser() { }

        // Using an pipeline-overridable constant.
        @id(42) override block_width = 12u;
        @compute @workgroup_size(block_width)
        fn shuffler() { }

        // Error: workgroup_size must be specified on compute shader
        @compute
        fn bad_shader() { }
        ```
- User defined Input and Output (which are shared between, shaders/stages) need a location or builtin attribute. That's why a struct, that is returned from a shader, always needs all it's members to have a location or builtin.
    -  A location can only store up to 16 bytes (max: vec4)
    ```rust
    struct A {
        @location(0) x: f32,
        // Despite locations being 16-bytes, x and y cannot share a location
        @location(1) y: f32
    }

    // in1 occupies locations 0 and 1.
    // in2 occupies location 2.
    // The return value occupies location 0.
    @fragment
    fn fragShader(in1: A, @location(2) in2: f32) -> @location(0) vec4<f32> {
    // ...
    }
    ```
    ```rust
    // Mixed builtins and user-defined inputs.
    struct MyInputs {
        @location(0) x: vec4<f32>,
        @builtin(front_facing) y: bool,
        @location(1) @interpolate(flat) z: u32
    }

    struct MyOutputs {
        @builtin(frag_depth) x: f32,
        @location(0) y: vec4<f32>
    }

    @fragment
    fn fragShader(in1: MyInputs) -> MyOutputs {
    // ...
    }
    ```
    ```rust
    struct A {
        @location(0) x: f32,
        // Invalid, x and y cannot share a location.
        @location(0) y: f32
    }

    struct B {
        @location(0) x: f32
    }

    struct C {
        // Invalid, structures with user-defined IO cannot be nested.
        b: B
    }

    struct D {
        x: vec4<f32>
    }

    @fragment
    // Invalid, location cannot be applied to a structure type.
    fn fragShader1(@location(0) in1: D) {
    // ...
    }

    @fragment
    // Invalid, in1 and in2 cannot share a location.
    fn fragShader2(@location(0) in1: f32, @location(0) in2: f32) {
    // ...
    }

    @fragment
    // Invalid, location cannot be applied to a structure.
    fn fragShader3(@location(0) in1: vec4<f32>) -> @location(0) D {
    // ...
    }
    ```
- u32 can represent 4294967295 but f32 convertion to u32 can only represent 4294967040
- Memory Model follows Vulkan Memory Model
- Atomics built-in functions ordering is relaxed. [more](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#memory-model-memory-semantics)
- workgroupBarrier uses AcquireRelease memory semantics and WorkgroupMemory semantics. 
- storageBarrier uses AcquireRelease memory semantics and UniformMemory semantics.

## Fragment Shaders
- Fragment shader invocations operating on neighbouring fragments (in screen-space coordinates) collaborate to compute approximate partial derivatives. These neighbouring fragments are referred to as a quad.

### partial derivative: 
the rate of change of a value along an axis. 
- implicit in functions: 
    - textureSample
    - textureSampleBias
    - textureSampleCompare.
- explicit in functions:
    - dpdx, dpdxCoarse, and dpdxFine compute partial derivatives along the x axis.
    - dpdy, dpdyCoarse, and dpdyFine compute partial derivatives along the y axis.
    - fwidth, fwidthCoarse, and fwidthFine compute the Manhattan metric over the associated x and y partial derivatives.

## Compute Shaders

- share access to shader variable in workgroup address space
- workgroup grid:
    - 0 ≤ i < workgroup_size_x
    - 0 ≤ j < workgroup_size_y
    - 0 ≤ k < workgroup_size_z
- each invocation gets a 'local invocation ID'
    - index calculates as following: 
        ```
        local_invocation_index = i + (j * workgroup_size_x) + (k * workgroup_size_x * workgroup_size_y)
        ```

- shader grid: 
    - 0 ≤ CSi < workgroup_size_x × group_count_x
    - 0 ≤ CSj < workgroup_size_y × group_count_y
    - 0 ≤ CSk < workgroup_size_z × group_count_z

- control barrier: excutes everthing in working group as if it were executed concurrently
    
## Built-in
### Built-in Values
|Name|Stage|Input or Output|Type|
|----|-----|---------------|----|
|vertex_index|vertex|input|`u32`|
|instance_index|vertex|input|`u32`|
|position|vertex|output|`vec4<f32>`|
|position|fragment|input|`vec4<f32>`|
|front_facing|fragment|input|`bool`|
|frag_depth|fragment|output|`f32`|
|local_invocation_id|compute|input|`vec3<u32>`|
|local_invocation_index|compute|input|`u32`|
|global_invocation_id|compute|input|`vec3<u32>`|
|workgroup_id|compute|input|`vec3<u32>`|
|num_workgroups|compute|input|`vec3<u32>`|
|sample_index|fragment|input|`u32`|
|sample_mask|fragment|input|`u32`|
|sample_mask|fragment|output|`u32`|
Example:
```rust
 struct VertexOutput {
   @builtin(position) my_pos: vec4<f32>
 }

 @vertex
 fn vs_main(
   @builtin(vertex_index) my_index: u32,
   @builtin(instance_index) my_inst_index: u32,
 ) -> VertexOutput {}

 struct FragmentOutput {
   @builtin(frag_depth) depth: f32,
   @builtin(sample_mask) mask_out: u32
 }

 @fragment
 fn fs_main(
   @builtin(front_facing) is_front: bool,
   @builtin(position) coord: vec4<f32>,
   @builtin(sample_index) my_sample_index: u32,
   @builtin(sample_mask) mask_in: u32,
 ) -> FragmentOutput {}

 @compute
 fn cs_main(
   @builtin(local_invocation_id) local_id: vec3<u32>,
   @builtin(local_invocation_index) local_index: u32,
   @builtin(global_invocation_id) global_id: vec3<u32>,
) {}
```

### Built-in Functions
- [Logical Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#logical-builtin-functions)
- [Array Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#array-builtin-functions)
- [Float Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#float-builtin-functions)
- [Integer Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#integer-builtin-functions)
- [Matrix Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#matrix-builtin-functions)
- [Vector Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#vector-builtin-functions)
- [Derivative Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#derivative-builtin-functions)
    - Must only be used in a fragment shader stage.
    - Must only be invoked in uniform control flow.
- [Texture Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#texture-builtin-functions)
- [Atomic Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#atomic-builtin-functions)
    - Atomic built-in functions must not be used in a vertex shader stage.
    - The address space SC of the atomic_ptr parameter in all atomic built-in functions must be either storage or workgroup.
- [Data Packing Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#pack-builtin-functions)
- [Data Unpacking Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#unpack-builtin-functions)
- [Synchronization Built-in Functions](https://gpuweb.github.io/gpuweb/wgsl/#sync-builtin-functions)
    - All synchronization functions execute a control barrier with Acquire/Release memory ordering.
    - All synchronization functions must only be used in the compute shader stage.




