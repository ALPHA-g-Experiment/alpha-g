# `alpha-g-wire-gain-calibration`

Gain calibration of the anode wire signals. Run the
`alpha-g-wire-gain-calibration --help` command to make sure you have installed
the `alpha-g-analysis` package and print help information.

## Requirements

Please make sure you have all the following installed in your system, otherwise
`alpha-g-wire-gain-calibration` may not work as expected:

- pdflatex: Access to the `pdflatex` command. Additionally, the `standalone` and
`pgfplots` packages are required to generate the plots. To make sure that these
are installed properly, run:
	```
	pdflatex "\documentclass{standalone}\usepackage{pgfplots}\begin{document}\begin{tikzpicture}\begin{axis}\end{axis}\end{tikzpicture}\end{document}"
	```
	If successful, this will create a PDF file with an empty plot in the current
working directory.
- Default PDF viewer: Use `xdg-mime` to configure the appropriate default PDF
 viewer ([example instructions for zathura](https://wiki.archlinux.org/title/zathura#Make_zathura_the_default_pdf_viewer)).
To make sure that this is configured correctly run:
	```
	xdg-open any_pdf_document_on_your_system.pdf
	```
	This should open the PDF with the desired viewer.
