# `alpha-g-wire-baseline-calibration`

Generate a calibration file with the baseline of all anode wire channels. Run
the `alpha-g-wire-baseline-calibration --help` command to make sure you have
installed the `alpha-g-analysis` package and print help information.

## Requirements

All input MIDAS files should belong to the same run with the following settings:
- Pulser enabled.
- Field wire pulser disabled.
- BSC pulser disabled.
- Trigger pulser must be enabled.
- There should be no other active trigger sources.
- Anode wire data suppression must be disabled.

Additionally, please make sure you have all the following installed in your
system, otherwise `alpha-g-wire-baseline-calibration` may not work as expected:

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
