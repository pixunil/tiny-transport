export class ProgramInfo {
    async setUp(gl, assets) {
        this.gl = gl;

        await this.loadProgram(assets);
        this.fetchLocations();
    }

    async loadProgram(assets) {
        const shaders = [
            this.loadShader(this.gl.VERTEX_SHADER, await assets.vertex),
            this.loadShader(this.gl.FRAGMENT_SHADER, await assets.fragment),
        ];

        this.program = this.gl.createProgram();
        for (const shader of shaders) {
            this.gl.attachShader(this.program, shader);
        }
        this.gl.linkProgram(this.program);

        if (!this.gl.getProgramParameter(this.program, this.gl.LINK_STATUS)) {
            const log = this.gl.getProgramInfoLog(this.program);
            this.gl.deleteProgram(this.program);
            throw {
                shaders: shaders,
                log: log,
            };
        }
    }

    loadShader(type, source) {
        const shader = this.gl.createShader(type);
        this.gl.shaderSource(shader, source);
        this.gl.compileShader(shader);

        if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
            const log = this.gl.getShaderInfoLog(shader);
            this.gl.deleteShader(shader);
            throw {
                type: type,
                source: source,
                log: log,
            };
        }

        return shader;
    }

    fetchLocations() {
        this.uniformLocations = {};
        const uniformCount = this.gl.getProgramParameter(this.program, this.gl.ACTIVE_UNIFORMS);
        for (let uniformIndex = 0; uniformIndex < uniformCount; uniformIndex++) {
            const uniform = this.gl.getActiveUniform(this.program, uniformIndex);
            const name = uniform.name.substr(2);
            this.uniformLocations[name] = this.gl.getUniformLocation(this.program, uniform.name);
        }

        this.attributeLocations = {};
        const attributeCount = this.gl.getProgramParameter(this.program, this.gl.ACTIVE_ATTRIBUTES);
        for (let attributeIndex = 0; attributeIndex < attributeCount; attributeIndex++) {
            const attribute = this.gl.getActiveAttrib(this.program, attributeIndex);
            const name = attribute.name.substr(2);
            this.attributeLocations[name] = this.gl.getAttribLocation(this.program, attribute.name);
        }
    }
}
