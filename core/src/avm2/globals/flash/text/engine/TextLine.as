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
            stub_method("flash.text.engine.TextLine", "getBaselinePosition");
            return 0.0;
        }

        public native function get hasTabs():Boolean;

        public native function getAtomIndexAtPoint(stageX:Number, stageY:Number):int;

        public native function getAtomIndexAtCharIndex(charIndex:int):int;

        public function getAtomBidiLevel(index:int):int {
            stub_method("flash.text.engine.TextLine", "getAtomBidiLevel");
            return 0;
        }

        public native function getAtomBounds(index:int):Rectangle;

        public native function getAtomCenter(index:int):Number;

        public function getAtomGraphic(index:int):DisplayObject {
            stub_method("flash.text.engine.TextLine", "getAtomGraphic");
            return null;
        }

        public native function getAtomTextBlockBeginIndex(index:int):int;

        public native function getAtomTextBlockEndIndex(index:int):int;

        public function getAtomTextRotation(index:int):String {
            stub_method("flash.text.engine.TextLine", "getAtomTextRotation");
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
