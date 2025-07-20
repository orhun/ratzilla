export class RatzillaCanvas {
    constructor() {}

    measure_text(text) {
        return this.ctx.measureText(text);
    }

    get_canvas() {
        return this.canvas;
    }

    reinit_canvas() {
        this.canvas.width = this.parent.clientWidth;
        this.canvas.height = this.parent.clientHeight;
        return new Uint16Array([this.canvas.width, this.canvas.height]);
    }
}
