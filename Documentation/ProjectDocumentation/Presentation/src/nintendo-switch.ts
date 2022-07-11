import { PDFViewer } from 'pdfjs-dist/web/pdf_viewer';

export const createLeftListener = (pdfViewer: PDFViewer) => ({
	0: () => pdfViewer.previousPage(), // Left Arrow
	3: () => pdfViewer.nextPage(), // Right Arrow

	4: () => pdfViewer.previousPage(), // SL
	5: () => pdfViewer.nextPage(), // SR

	6: () => pdfViewer.previousPage(), // ZL
	8: () => pdfViewer.nextPage(), // L

	16: async () => await document.exitFullscreen() // Rectangle
});

export const createRightListener = (pdfViewer: PDFViewer) => ({
	0: () => pdfViewer.nextPage(), // A
	3: () => pdfViewer.previousPage(), // Y

	4: () => pdfViewer.nextPage(), // SL
	5: () => pdfViewer.previousPage(), // SR

	7: () => pdfViewer.previousPage(), // ZR
	8: () => pdfViewer.nextPage(), // R

	16: async () => await document.exitFullscreen() // Home
});
