package flash.text.engine {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_method;

    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.errors.IllegalOperationError;
    import flash.geom.Rectangle;
    import flash.ui.ContextMenu;

    // FIXME: None of the DisplayObjectContainer methods actually work on
    // the TextLine class in Ruffle, despite the methods working fine in FP-
    // however, it's unlikely that SWFs will actually attempt to add children
    // to a TextLine.
    [Ruffle(Abstract)]
    [API("662")]
    public final class TextLine extends DisplayObjectContainer {
        public static const MAX_LINE_WIDTH:int = 1000000;

        public var userData;

        public native function get rawTextLength():int;

        public native function get textBlockBeginIndex():int;

        public native function get specifiedWidth():Number;

        public native function get textBlock():TextBlock;

        public native function get ascent():Number;

        [API("670")]
        public function get totalAscent():Number {
            stub_getter("flash.text.engine.TextLine", "totalAscent");
            return this.ascent;
        }

        public native function get descent():Number;

        [API("670")]
        public function get totalDescent():Number {
            stub_getter("flash.text.engine.TextLine", "totalDescent");
            return this.descent;
        }

        public function get unjustifiedTextWidth():Number {
            // Ruffle does not justify lines yet.
            return this.textWidth;
        }

        public native function get textWidth():Number;
        public native function get textHeight():Number;

        public native function get validity():String;
        public native function set validity(value:String):void;

        public function get hasGraphicElement():Boolean {
            stub_getter("flash.text.engine.TextLine", "hasGraphicElement");
            return false;
        }

        public native function get atomCount():int;

        public native function get previousLine():TextLine;
        public native function get nextLine():TextLine;

        public function getBaselinePosition(baseline:String):Number {
            // FIXME: This assumes that the dominant baseline is the roman one.
            switch (baseline) {
                case TextBaseline.ROMAN:
                    return 0.0;
                case TextBaseline.ASCENT:
                    return -this.ascent;
                case TextBaseline.DESCENT:
                    return this.descent;
                case TextBaseline.IDEOGRAPHIC_TOP:
                    stub_method("flash.text.engine.TextLine", "getBaselinePosition", "with ideographic baselines");
                    return -this.ascent;
                case TextBaseline.IDEOGRAPHIC_CENTER:
                    stub_method("flash.text.engine.TextLine", "getBaselinePosition", "with ideographic baselines");
                    return (this.descent - this.ascent) / 2;
                case TextBaseline.IDEOGRAPHIC_BOTTOM:
                    stub_method("flash.text.engine.TextLine", "getBaselinePosition", "with ideographic baselines");
                    return this.descent;
                default:
                    Error.throwError(ArgumentError, 2008, "baseline");
                    return 0.0;
            }
        }

        public native function get hasTabs():Boolean;

        public native function getAtomIndexAtPoint(stageX:Number, stageY:Number):int;

        public native function getAtomIndexAtCharIndex(charIndex:int):int;

        private function checkAtomIndex(index:int):void {
            if (index < 0 || index >= this.atomCount) {
                Error.throwError(RangeError, 2006);
            }
        }

        public function getAtomBidiLevel(index:int):int {
            this.checkAtomIndex(index);
            // Ruffle doesn't support bidirectional text: all atoms are laid
            // out left-to-right, at the paragraph embedding level.
            var level:int = this.textBlock != null ? this.textBlock.bidiLevel : 0;
            if (level % 2 != 0) {
                stub_method("flash.text.engine.TextLine", "getAtomBidiLevel", "with right-to-left text");
            }
            return level;
        }

        public native function getAtomBounds(index:int):Rectangle;

        public native function getAtomCenter(index:int):Number;

        public function getAtomGraphic(index:int):DisplayObject {
            this.checkAtomIndex(index);
            if (this.hasGraphicElement) {
                stub_method("flash.text.engine.TextLine", "getAtomGraphic", "with graphic elements");
            }
            return null;
        }

        public native function getAtomTextBlockBeginIndex(index:int):int;

        public native function getAtomTextBlockEndIndex(index:int):int;

        public function getAtomTextRotation(index:int):String {
            this.checkAtomIndex(index);
            // Text rotation isn't supported by Ruffle's line layout.
            return TextRotation.ROTATE_0;
        }

        public native function getAtomWordBoundaryOnLeft(index:int):Boolean;

        // This function does nothing in Flash Player 32
        public function flushAtomData():void { }

        // Overrides

        override public function set contextMenu(cm:ContextMenu):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set focusRect(value:Object):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set tabChildren(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set tabEnabled(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set tabIndex(index:int):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        // End of overrides
    }
}
