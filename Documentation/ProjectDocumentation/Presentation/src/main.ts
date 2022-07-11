import { getDocument, GlobalWorkerOptions, PDFDocumentProxy } from 'pdfjs-dist';
import {
	PDFViewer,
	EventBus,
	PDFLinkService,
	PDFFindController,
	PDFScriptingManager,
	GenericL10n
} from 'pdfjs-dist/web/pdf_viewer';

// @ts-ignore
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';
// @ts-ignore
import SANDBOX_BUNDLE_SRC from 'pdfjs-dist/build/pdf.sandbox.js?url';
import { createLeftListener, createRightListener } from './nintendo-switch';
// https://www.smashingmagazine.com/2015/11/gamepad-api-in-web-games/

type GamepadId = string;

async function main() {
	GlobalWorkerOptions.workerSrc = workerSrc;

	const fullscreenBtn = <HTMLButtonElement>document.getElementById('fullscreenBtn');
	const container = <HTMLDivElement>document.getElementById('viewerContainer');
	const viewer = <HTMLDivElement>document.getElementById('viewer');

	const eventBus = new EventBus();
	const l10n = new GenericL10n('en');
	const linkService = new PDFLinkService({ eventBus });
	const findController = new PDFFindController({ eventBus, linkService });
	const scriptingManager = new PDFScriptingManager({
		eventBus,
		sandboxBundleSrc: SANDBOX_BUNDLE_SRC
	});

	const pdfViewer = new PDFViewer({
		eventBus,
		viewer,
		container,
		l10n,
		linkService,
		findController,
		scriptingManager,
		renderer: 'canvas'
	});

	linkService.setViewer(pdfViewer);
	scriptingManager.setViewer(pdfViewer);

	const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);
	if (isSafari) console.log('fucking useless browser');
	const a_index = !isSafari ? 1 : 0;
	const y_index = !isSafari ? 2 : 3;

	const DEFAULT_LISTENER = {
		// A
		[a_index]: () => {
			pdfViewer.nextPage();
			console.debug(`${a_index}, A`);
		},
		// Y
		[y_index]: () => {
			pdfViewer.previousPage();
			console.debug(`${y_index}, Y`);
		},

		// L
		4: () => {
			pdfViewer.nextPage();
			console.debug('4, L');
		},
		// ZL
		6: () => {
			pdfViewer.previousPage();
			console.debug('6, ZL');
		},

		// R
		5: () => {
			pdfViewer.nextPage();
			console.debug('5, R');
		},
		// ZR
		7: () => {
			pdfViewer.previousPage();
			console.debug('7, ZR');
		},

		// Left Arrow
		14: () => {
			pdfViewer.previousPage();
			console.debug('14, Left Arrow');
		},
		// Right Arrow
		15: () => {
			pdfViewer.nextPage();
			console.debug('15, Right Arrow');
		},

		// Home
		16: async () => await document.exitFullscreen(),
		// Rectangle
		17: async () => await document.exitFullscreen()
	};

	eventBus.on('pagesinit', function () {
		// We can use pdfSinglePageViewer now, e.g. let's change default scale.
		pdfViewer.currentScaleValue = 'page-width';
	});

	const pdfDoc = await getDocument('/presentation/presentation.pdf').promise;

	pdfViewer.setDocument(pdfDoc);
	linkService.setDocument(pdfDoc, null);

	const presentationMode = new PDFPresentationMode({
		container,
		eventBus,
		pdfViewer
	});

	window.addEventListener('resize', webViewerResize);
	window.addEventListener('keydown', webViewerKeyDown);

	fullscreenBtn.addEventListener('click', async () => {
		eventBus.dispatch('presentationmode', { source: fullscreenBtn });
	});

	eventBus._on('presentationmodechanged', webViewerPresentationModeChanged);
	eventBus._on('presentationmode', webViewerPresentationMode);

	type PageChangeEvent = {
		source: { pdfDocument: PDFDocumentProxy; _pages: { div: HTMLDivElement }[] };
		previous: number;
		pageNumber: number;
	};

	// console.debug(await pdfDoc.hasJSActions());
	// console.debug(await pdfDoc.getJSActions());
	// console.debug(await pdfDoc.getJavaScript());
	// const objects = await pdfDoc.getFieldObjects();
	// console.debug(objects);

	eventBus._on('pagechanging', async (e: PageChangeEvent) => {
		// const p = await pdfDoc.getPage(e.pageNumber);
		// const structTree = await p.getStructTree();
		// const actions = await p.getJSActions();
		// console.debug(structTree);
		// console.debug(actions);

		const prevPage = e.source._pages[e.previous - 1].div as HTMLDivElement;
		const gifs = Array.from(prevPage.getElementsByClassName('gif'));
		gifs.forEach((gif) => gif.remove());

		const page = e.source._pages[e.pageNumber - 1].div as HTMLDivElement;
		const anchors = Array.from(page.getElementsByTagName('a'));

		anchors.forEach((anchor, i) => {
			if (!anchor.href.endsWith('.gif')) return;
			const img = document.createElement('img');
			img.src = anchor.href;
			img.style.width = '100%';
			img.style.height = '100%';
			img.style.zIndex = '1';
			img.classList.add('gif');
			anchor.parentNode?.appendChild(img);
		});
	});

	window.addEventListener('gamepadconnected', webViewerGamepadConnected);
	window.addEventListener('gamepaddisconnected', webViewerGamepadDisconnected);

	const gamepadListeners = new Map<GamepadId, Record<number, () => any | Promise<any>>>();
	gamepadButtonPress(gamepadListeners);

	function webViewerGamepadConnected(e: GamepadEvent) {
		const { gamepad } = e;

		let listeners: Record<number, () => any | Promise<any>> = DEFAULT_LISTENER;
		switch (gamepad.id) {
			case 'Joy-Con (L) Gamepad': {
				listeners = createRightListener(pdfViewer);
				break;
			}
			case '57e-2006-Joy-Con (L)': {
				listeners = createLeftListener(pdfViewer);
				break;
			}
			case 'Joy-Con (L) (STANDARD GAMEPAD Vendor: 057e Product: 2006)': {
				listeners = createLeftListener(pdfViewer);
				break;
			}
			case 'Joy-Con (R) Gamepad': {
				listeners = createRightListener(pdfViewer);
				break;
			}
			case '57e-2007-Joy-Con (R)': {
				listeners = createRightListener(pdfViewer);
				break;
			}
			case 'Joy-Con (R) (STANDARD GAMEPAD Vendor: 057e Product: 2007)': {
				listeners = createRightListener(pdfViewer);
				break;
			}
			default: {
				console.debug('unknown Controller using default Layout');
				break;
			}
		}

		gamepadListeners.set(gamepad.id, listeners);
	}

	function webViewerGamepadDisconnected(e: GamepadEvent) {
		const { gamepad } = e;
		gamepadListeners.delete(gamepad.id);
	}

	async function gamepadButtonPress(
		gamepadListeners: Map<GamepadId, Record<number, () => any | Promise<any>>>
	) {
		const lastStates: boolean[][] = [];

		while (true) {
			const gamepads = <Gamepad[]>[...navigator.getGamepads()].filter(Boolean);

			if (gamepads.length === 0) {
				console.debug('waiting for conntection');
				await new Promise<void>((resolve) => {
					addEventListener('gamepadconnected', () => resolve(), { once: true });
				});
				continue;
			}

			await new Promise((r) => requestAnimationFrame(r));

			for (const gamepad of gamepads) {
				const state = gamepad.buttons.map((button) => button.pressed);
				const lastState = lastStates[gamepad.index] || state.map(() => false);

				const listeners = gamepadListeners.get(gamepad.id) ?? DEFAULT_LISTENER;

				for (const [buttonIndex, callback] of Object.entries(listeners)) {
					const wasPressed = lastState[buttonIndex];
					const pressed = state[buttonIndex];
					if (pressed && !wasPressed) await callback();
				}

				lastStates[gamepad.index] = state;
			}
		}
	}

	function webViewerPresentationModeChanged(evt) {
		pdfViewer.presentationModeState = evt.state;
	}

	async function webViewerPresentationMode() {
		if (!(await presentationMode.request())) {
			alert(`Presentationmode not supported on IOS, please complaint to Apple`);
		}
	}

	function webViewerResize() {
		pdfViewer.updateContainerHeightCss();
		pdfViewer.currentScaleValue = pdfViewer.currentScaleValue;
		pdfViewer.update();
	}

	function webViewerKeyDown(event: KeyboardEvent) {
		switch (event.key) {
			case 'ArrowLeft': {
				pdfViewer.previousPage();
				break;
			}
			case 'ArrowRight': {
				pdfViewer.nextPage();
				break;
			}
			case 'ArrowUp': {
				pdfViewer.previousPage();
				break;
			}
			case 'ArrowDown': {
				pdfViewer.nextPage();
			}
		}
	}
}

main();

const PresentationModeState = {
	UNKNOWN: 0,
	NORMAL: 1,
	CHANGING: 2,
	FULLSCREEN: 3
};

const SpreadMode = {
	UNKNOWN: -1,
	NONE: 0, // Default value.
	ODD: 1,
	EVEN: 2
};

const enum ScrollMode {
	UNKNOWN = -1,
	VERTICAL = 0, // Default value.
	HORIZONTAL = 1,
	WRAPPED = 2,
	PAGE = 3
}

const DELAY_BEFORE_HIDING_CONTROLS = 3000; // in ms
const ACTIVE_SELECTOR = 'pdfPresentationMode';
const CONTROLS_SELECTOR = 'pdfPresentationModeControls';

// Number of CSS pixels for a movement to count as a swipe.
const SWIPE_MIN_DISTANCE_THRESHOLD = 50;

// Swipe angle deviation from the x or y axis before it is not
// considered a swipe in that direction any more.
const SWIPE_ANGLE_THRESHOLD = Math.PI / 6;

interface Args {
	pageNumber: number;
	scaleValue: string;
	scrollMode: ScrollMode;
	spreadMode: number | null;
}

type PDFPresentationModeOptions = {
	container: HTMLDivElement;
	pdfViewer: import('pdfjs-dist/web/pdf_viewer').PDFViewer;
	eventBus: import('pdfjs-dist/web/pdf_viewer').EventBus;
};

class PDFPresentationMode {
	#state = PresentationModeState.UNKNOWN;

	#args: Args | null = null;
	container: HTMLDivElement;
	pdfViewer: PDFViewer;
	eventBus: EventBus;
	contextMenuOpen: boolean;
	mouseScrollTimeStamp: number;
	mouseScrollDelta: number;
	touchSwipeState: any | null;
	controlsTimeout: any;
	showControlsBind: any;
	mouseDownBind: any;
	resetMouseScrollStateBind: any;
	contextMenuBind: any;
	touchSwipeBind: any;
	fullscreenChangeBind: any;

	constructor({ container, pdfViewer, eventBus }: PDFPresentationModeOptions) {
		this.container = container;
		this.pdfViewer = pdfViewer;
		this.eventBus = eventBus;

		this.contextMenuOpen = false;
		this.mouseScrollTimeStamp = 0;
		this.mouseScrollDelta = 0;
		this.touchSwipeState = null;
	}

	/**
	 * Request the browser to enter fullscreen mode.
	 * @returns {Promise<boolean>} Indicating if the request was successful.
	 */
	async request(): Promise<boolean> {
		const { container, pdfViewer } = this;

		container.requestFullscreen =
			// @ts-ignore
			container.requestFullscreen || container.webkitRequestFullScreen;

		if (this.active || !pdfViewer.pagesCount || !container.requestFullscreen) {
			return false;
		}
		this.#addFullscreenChangeListeners();
		this.#notifyStateChange(PresentationModeState.CHANGING);

		const promise = container.requestFullscreen();

		this.#args = {
			pageNumber: pdfViewer.currentPageNumber,
			scaleValue: pdfViewer.currentScaleValue,
			scrollMode: pdfViewer.scrollMode,
			spreadMode: null
		};

		if (
			pdfViewer.spreadMode !== SpreadMode.NONE &&
			!(pdfViewer.pageViewsReady && pdfViewer.hasEqualPageSizes)
		) {
			console.warn(
				'Ignoring Spread modes when entering PresentationMode, ' +
					'since the document may contain varying page sizes.'
			);
			this.#args.spreadMode = pdfViewer.spreadMode;
		}

		try {
			await promise;
			return true;
		} catch (reason) {
			this.#removeFullscreenChangeListeners();
			this.#notifyStateChange(PresentationModeState.NORMAL);
		}
		return false;
	}

	get active() {
		return (
			this.#state === PresentationModeState.CHANGING ||
			this.#state === PresentationModeState.FULLSCREEN
		);
	}

	#notifyStateChange(state) {
		this.#state = state;

		this.eventBus.dispatch('presentationmodechanged', { source: this, state });
	}

	#enter() {
		this.#notifyStateChange(PresentationModeState.FULLSCREEN);
		this.container.classList.add(ACTIVE_SELECTOR);

		// Ensure that the correct page is scrolled into view when entering
		// Presentation Mode, by waiting until fullscreen mode in enabled.
		setTimeout(() => {
			this.pdfViewer.scrollMode = ScrollMode.PAGE;
			if (this.#args?.spreadMode !== null) this.pdfViewer.spreadMode = SpreadMode.NONE;
			this.pdfViewer.currentPageNumber = this.#args?.pageNumber ?? 1;
			this.pdfViewer.currentScaleValue = 'page-fit';
		}, 0);

		this.#addWindowListeners();
		this.#showControls();
		this.contextMenuOpen = false;

		// Text selection is disabled in Presentation Mode, thus it's not possible
		// for the user to deselect text that is selected (e.g. with "Select all")
		// when entering Presentation Mode, hence we remove any active selection.
		window.getSelection()?.removeAllRanges();
	}

	#exit() {
		const pageNumber = this.pdfViewer.currentPageNumber;
		this.container.classList.remove(ACTIVE_SELECTOR);

		// Ensure that the correct page is scrolled into view when exiting
		// Presentation Mode, by waiting until fullscreen mode is disabled.
		setTimeout(() => {
			this.#removeFullscreenChangeListeners();
			this.#notifyStateChange(PresentationModeState.NORMAL);

			this.pdfViewer.scrollMode = this.#args!!.scrollMode;
			this.pdfViewer.spreadMode = this.#args?.spreadMode ?? this.pdfViewer.spreadMode;
			this.pdfViewer.currentScaleValue = this.#args?.scaleValue ?? 'page-width';
			this.pdfViewer.currentPageNumber = pageNumber;
			this.#args = null;
		}, 0);

		this.#removeWindowListeners();
		this.#hideControls();
		this.#resetMouseScrollState();
		this.contextMenuOpen = false;
	}

	#mouseDown(evt) {
		if (this.contextMenuOpen) {
			this.contextMenuOpen = false;
			evt.preventDefault();
			return;
		}
		if (evt.button === 0) {
			// Enable clicking of links in presentation mode. Note: only links
			// pointing to destinations in the current PDF document work.
			const isInternalLink = evt.target.href && evt.target.classList.contains('internalLink');
			if (!isInternalLink) {
				// Unless an internal link was clicked, advance one page.
				evt.preventDefault();

				if (evt.shiftKey) {
					this.pdfViewer.previousPage();
				} else {
					this.pdfViewer.nextPage();
				}
			}
		}
	}

	#contextMenu() {
		this.contextMenuOpen = true;
	}

	#showControls() {
		if (this.controlsTimeout) {
			clearTimeout(this.controlsTimeout);
		} else {
			this.container.classList.add(CONTROLS_SELECTOR);
		}
		this.controlsTimeout = setTimeout(() => {
			this.container.classList.remove(CONTROLS_SELECTOR);
			delete this.controlsTimeout;
		}, DELAY_BEFORE_HIDING_CONTROLS);
	}

	#hideControls() {
		if (!this.controlsTimeout) {
			return;
		}
		clearTimeout(this.controlsTimeout);
		this.container.classList.remove(CONTROLS_SELECTOR);
		delete this.controlsTimeout;
	}

	/**
	 * Resets the properties used for tracking mouse scrolling events.
	 */
	#resetMouseScrollState() {
		this.mouseScrollTimeStamp = 0;
		this.mouseScrollDelta = 0;
	}

	#touchSwipe(evt) {
		if (!this.active) {
			return;
		}
		if (evt.touches.length > 1) {
			// Multiple touch points detected; cancel the swipe.
			this.touchSwipeState = null;
			return;
		}

		switch (evt.type) {
			case 'touchstart':
				this.touchSwipeState = {
					startX: evt.touches[0].pageX,
					startY: evt.touches[0].pageY,
					endX: evt.touches[0].pageX,
					endY: evt.touches[0].pageY
				};
				break;
			case 'touchmove':
				if (this.touchSwipeState === null) {
					return;
				}
				this.touchSwipeState.endX = evt.touches[0].pageX;
				this.touchSwipeState.endY = evt.touches[0].pageY;
				// Avoid the swipe from triggering browser gestures (Chrome in
				// particular has some sort of swipe gesture in fullscreen mode).
				evt.preventDefault();
				break;
			case 'touchend':
				if (this.touchSwipeState === null) {
					return;
				}
				let delta = 0;
				const dx = this.touchSwipeState.endX - this.touchSwipeState.startX;
				const dy = this.touchSwipeState.endY - this.touchSwipeState.startY;
				const absAngle = Math.abs(Math.atan2(dy, dx));
				if (
					Math.abs(dx) > SWIPE_MIN_DISTANCE_THRESHOLD &&
					(absAngle <= SWIPE_ANGLE_THRESHOLD ||
						absAngle >= Math.PI - SWIPE_ANGLE_THRESHOLD)
				) {
					// Horizontal swipe.
					delta = dx;
				} else if (
					Math.abs(dy) > SWIPE_MIN_DISTANCE_THRESHOLD &&
					Math.abs(absAngle - Math.PI / 2) <= SWIPE_ANGLE_THRESHOLD
				) {
					// Vertical swipe.
					delta = dy;
				}
				if (delta > 0) {
					this.pdfViewer.previousPage();
				} else if (delta < 0) {
					this.pdfViewer.nextPage();
				}
				break;
		}
	}

	#addWindowListeners() {
		this.showControlsBind = this.#showControls.bind(this);
		this.mouseDownBind = this.#mouseDown.bind(this);
		this.resetMouseScrollStateBind = this.#resetMouseScrollState.bind(this);
		this.contextMenuBind = this.#contextMenu.bind(this);
		this.touchSwipeBind = this.#touchSwipe.bind(this);

		window.addEventListener('mousemove', this.showControlsBind);
		window.addEventListener('mousedown', this.mouseDownBind);
		window.addEventListener('keydown', this.resetMouseScrollStateBind);
		window.addEventListener('contextmenu', this.contextMenuBind);
		window.addEventListener('touchstart', this.touchSwipeBind);
		window.addEventListener('touchmove', this.touchSwipeBind);
		window.addEventListener('touchend', this.touchSwipeBind);
	}

	#removeWindowListeners() {
		window.removeEventListener('mousemove', this.showControlsBind);
		window.removeEventListener('mousedown', this.mouseDownBind);
		window.removeEventListener('keydown', this.resetMouseScrollStateBind);
		window.removeEventListener('contextmenu', this.contextMenuBind);
		window.removeEventListener('touchstart', this.touchSwipeBind);
		window.removeEventListener('touchmove', this.touchSwipeBind);
		window.removeEventListener('touchend', this.touchSwipeBind);

		delete this.showControlsBind;
		delete this.mouseDownBind;
		delete this.resetMouseScrollStateBind;
		delete this.contextMenuBind;
		delete this.touchSwipeBind;
	}

	#fullscreenChange() {
		if (/* isFullscreen = */ document.fullscreenElement) {
			this.#enter();
		} else {
			this.#exit();
		}
	}

	#addFullscreenChangeListeners() {
		this.fullscreenChangeBind = this.#fullscreenChange.bind(this);
		window.addEventListener('fullscreenchange', this.fullscreenChangeBind);
		window.addEventListener('webkitfullscreenchange', this.fullscreenChangeBind);
	}

	#removeFullscreenChangeListeners() {
		window.removeEventListener('fullscreenchange', this.fullscreenChangeBind);
		window.removeEventListener('webkitfullscreenchange', this.fullscreenChangeBind);

		delete this.fullscreenChangeBind;
	}
}

export { PDFPresentationMode };
