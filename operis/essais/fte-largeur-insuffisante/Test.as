package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        var fmt:ElementFormat = new ElementFormat(new FontDescription("Noto Sans"), 12);
        var texts:Array = ["...", "…", "Texte long", "i", " x"];
        var widths:Array = [0, 1, 3, 5, 8, 10, 12, 20];
        for each (var t:String in texts) {
            var row:String = "[" + t + "]";
            for each (var w:Number in widths) {
                var block:TextBlock = new TextBlock(new TextElement(t, fmt));
                var n:int = 0;
                var line:TextLine = block.createTextLine(null, w);
                var first:String = block.textLineCreationResult;
                while (line) { n++; line = block.createTextLine(line, w); }
                row += " w" + w + ":" + n + "/" + first;
            }
            trace(row);
        }
    }
}
}
