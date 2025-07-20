export class RatzillaCanvas {
    constructor() {}

    measure_text(text) {
        let metrics = this.ctx.measureText(text);
        this.cellWidth = Math.floor(metrics.width);
        this.cellHeight = Math.floor(Math.abs(metrics.fontBoundingBoxAscent) + Math.abs(metrics.fontBoundingBoxDescent));
        this.cellAscent = Math.floor(metrics.fontBoundingBoxAscent);
        return new Float64Array([this.cellWidth, this.cellHeight, this.cellAscent]);
    }

    init_ctx() {
        this.ctx = this.canvas.getContext("2d", {
            alpha: true,
            desynchronized: true
        });
        this.ctx.font = this.font_str;
        this.ctx.textBaseline = "top";
    }

    get_canvas() {
        return this.canvas;
    }

    reinit_canvas() {
        let sourceW = Math.floor(this.parent.clientWidth / this.cellWidth);
        let sourceH = Math.floor(this.parent.clientHeight / this.cellHeight);

        let canvasW = sourceW * this.cellWidth;
        let canvasH = sourceH * this.cellHeight;

        if (this.canvas.width != canvasW || this.canvas.height != canvasH) {
            let dummyCanvas = new OffscreenCanvas(this.canvas.width, this.canvas.height);
            let dummyCtx = dummyCanvas.getContext('2d', {
                alpha: true
            });

            dummyCtx.drawImage(this.canvas, 0, 0);

            this.canvas.width = canvasW;
            this.canvas.height = canvasH;
            this.init_ctx();

            this.ctx.drawImage(dummyCanvas,
                0, 0,
                canvasW, canvasH,
                0, 0,
                canvasW, canvasH,
            );
        }
        
        return new Uint16Array([sourceW, sourceH]);
    }
}
