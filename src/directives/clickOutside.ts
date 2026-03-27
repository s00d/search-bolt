interface ExtendedHTMLElement extends HTMLElement {
  _clickOutside?: (event: Event) => void;
  _clickOutsideTimer?: number;
}

export const clickOutside = {
  mounted(el: ExtendedHTMLElement, binding: { value: () => void }) {
    el._clickOutside = (event: Event) => {
      if (!(el === event.target || el.contains(event.target as Node))) {
        binding.value();
      }
    };

    // Delay activation slightly to avoid immediate close on mount.
    el._clickOutsideTimer = window.setTimeout(() => {
      if (el._clickOutside) {
        document.addEventListener('mousedown', el._clickOutside);
      }
    }, 50);
  },

  unmounted(el: ExtendedHTMLElement) {
    if (el._clickOutsideTimer) {
      clearTimeout(el._clickOutsideTimer);
    }
    if (el._clickOutside) {
      document.removeEventListener('mousedown', el._clickOutside);
    }
  }
}
