import { createApp } from "vue";
import { clickOutside } from './directives/clickOutside'
import './style.css'
import App from "./App.vue";

const app = createApp(App)
app.directive('click-outside', clickOutside)
app.mount("#app");
