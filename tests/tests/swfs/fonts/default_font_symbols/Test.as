package {
import flash.display.*;
import flash.text.*;
import flash.text.engine.*;

[SWF(width="300", height="200")]
public class Test extends Sprite {
    // Characters often used as icons in button labels (Geometric Shapes
    // and Arrows). They are all in Arial (WGL4), but not in Noto Sans.
    private static const CHARS:Array = [0x25B2, 0x25BA, 0x25BC, 0x25C4, 0x25CF, 0x25A0, 0x2192, 0x2190];

    public function Test() {
        stage.scaleMode = "noScale";
        var y:Number = 0;
        for each (var code:int in CHARS) {
            var s:String = String.fromCharCode(code);

            var tf:TextField = new TextField();
            tf.defaultTextFormat = new TextFormat("Arial", 20);
            tf.autoSize = TextFieldAutoSize.LEFT;
            tf.text = "a" + s + "a";
            tf.y = y;
            addChild(tf);
            var bounds:* = tf.getCharBoundaries(1);
            var a:* = tf.getCharBoundaries(0);

            var block:TextBlock = new TextBlock();
            block.content = new TextElement(s, new ElementFormat(new FontDescription("Arial"), 20));
            var line:TextLine = block.createTextLine(null, 1000);
            line.x = 100;
            line.y = y + 20;
            addChild(line);

            trace("U+" + code.toString(16).toUpperCase() + ": TextField has width: " + (bounds != null && bounds.width > 2)
                + ", advances: " + (bounds != null && bounds.x > a.x + a.width - 1 && bounds.width > 2)
                + ", TextLine has width: " + (line.width > 2));
            y += 22;
        }
    }
}
}
