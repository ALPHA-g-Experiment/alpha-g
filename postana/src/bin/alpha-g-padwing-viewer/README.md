# `alpha-g-padwing-viewer`

Iterate through a MIDAS file, and visualize the individual cathode pad waveforms
from the radial Time Projection Chamber. Run the `alpha-g-padwing-viewer --help`
command to make sure you have installed the `alpha-g-postana` package and print 
help information.

## Requirements

Please make sure you have all the following installed in your system, otherwise
`alpha-g-padwing-viewer` may not work as expected:

- ncurses: This is the default backend used for the TUI. In the very unlikely
 scenario that you don't have this already, see [how to install
ncurses](https://github.com/gyscos/cursive/wiki/Install-ncurses).
- pdflatex: Access to the `pdflatex` command. Additionally, the `standalone` and
`pgfplots` packages are required to generate the plots. To make sure that these
are installed properly, run:
	```
	pdflatex "\documentclass{standalone}\usepackage{pgfplots}\begin{document}\begin{tikzpicture}\begin{axis}\end{axis}\end{tikzpicture}\end{document}"
	```
	If successful, this will create a PDF file with an empty plot in the current
working directory.
- PDF viewer: Install a PDF viewer which refreshes the output whenever a
 document is re-compiled. There are multiple viewers that support this feature;
all are equally valid. I personally use
[zathura](https://wiki.archlinux.org/title/zathura).
- Default viewer: To make sure that the correct PDF viewer is used to open the
plots, run:
	```
	xdg-open any_pdf_document_on_your_system.pdf
	```
	This should open the PDF with the viewer from the previous point. Otherwise,
use `xdg-mime` to configure the appropriate default PDF viewer. [Example
instructions for
zathura](https://wiki.archlinux.org/title/zathura#Make_zathura_the_default_pdf_viewer).
