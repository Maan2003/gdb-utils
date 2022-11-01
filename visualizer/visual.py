from typing import TextIO
import gdb
import graphviz
from graphviz.graphs import Digraph

def as_list(expr: gdb.Value) -> list[gdb.Value] | None:
    if vz := gdb.default_visualizer(expr):
        if vz.display_hint() == "array":
            return list(map(lambda x: x[1], vz.children()))
    # TODO: handle C array

class GraphViz(gdb.Command):
    """Visualize a graph into a dot file\nUsage: graph-viz EXPR FILE """
    def __init__(self) -> None:
        super(GraphViz, self).__init__("graph-viz", gdb.COMMAND_USER, gdb.COMPLETE_EXPRESSION)

    def invoke(self, argument: str, from_tty: bool) -> None:
        [expr, file] = argument.rsplit(' ', 1)
        expr = gdb.parse_and_eval(expr)
        g = as_list(expr) 
        if not g:
            print("cannot parse graph")
            return
        gr = Digraph()
        for u, vs in enumerate(g):
            gr.node(str(u))
            for v in as_list(vs):
                gr.edge(str(u), str(v))

        gr.render(outfile=file)

GraphViz()

tbl_style = """
<style>
  html,
  body {
    margin: 0;
    font-family: Input Mono;
  }
  table {
    border-collapse: collapse;
  }

  caption {
    margin-bottom: 10px;
  }

  .hl {
    background-color: cyan;
  }

  .heading {
    font-weight: 400;
    padding-right: 5px;
    padding-bottom: 2px;
  }
  td.data {
    border: black 1px solid;
    padding: 10px;
    min-width: 10px;
  }
</style>
"""

# draw a table
# TODO: 3d dp?
class TableViz(gdb.Command):
    """Visualize a 1D/2D array into html file\nUsage: tab-viz EXPR HI... FILE\n\nExample: tab-viz dp {0,0} """
    def __init__(self) -> None:
        super(TableViz, self).__init__("tab-viz", gdb.COMMAND_USER, gdb.COMPLETE_EXPRESSION)

    def invoke(self, argument: str, from_tty: bool) -> None:
        [expr_text, *his, file] = argument.split(' ')
        try:
            expr = gdb.parse_and_eval(expr_text)
        except:
            return
        a = as_list(expr) 
        if not a:
            print("cannot parse array")
            return
        
        def get_idx(expr: str):
            try:
                x = gdb.parse_and_eval(expr)
                if x.type.code == gdb.TYPE_CODE_INT:
                    return int(x)
                elif x.type.code == gdb.TYPE_CODE_ARRAY:
                    # TODO: handle multi dimesional array
                    return (int(x[0]), int(x[1]))
            except:
                pass

        his = {get_idx(x) for x in his}

        with open(file, "w") as f:
            f.write(tbl_style)
            f.write("<table>")
            f.write(f"<caption>{expr_text}</caption>")
            d1 = len(a)
            d2 = None
            for x in a:
                if val := as_list(x):
                    d2 = max(len(val), d2 or 0)
            if d2 is not None:
                mat = [as_list(x) for x in a]

                f.write("<tr>")
                f.write("<th></th>")
                for i in range(len(mat)):
                    f.write(f'<th class="heading">{i}</th>')
                f.write("</tr>")

                for j in range(d2):
                    f.write("<tr>")
                    f.write(f'<td class="heading">{j}</td>')
                    for i in range(d1):
                        val = str(mat[i][j]) if len(mat[i]) > j else ""
                        f.write(f'<td class="data{" hl" if (i, j) in his else ""}">{val}</td>')
                    f.write("</tr>")
            else:
                f.write("<tr>")
                for i in range(len(a)):
                    f.write(f'<th class="heading">{i}</th>')
                f.write("</tr>")

                f.write("<tr>")
                for (i, x) in enumerate(a):
                    f.write(f'<td class="data{" hl" if i in his else ""}">{x}</td>')
                    f.write("</tr>")
                f.write("</tr>")
            f.write("</table>")
TableViz()
