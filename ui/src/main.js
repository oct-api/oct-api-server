import { createApp } from 'vue'
import App from './App.vue'
import axios from 'axios'
import VueAxios from 'vue-axios'
import router from './router'
import VueApexCharts from "vue3-apexcharts";
import VueGtag from "vue-gtag-next";

const app = createApp(App);
app.use(VueAxios, axios).use(router);
app.use(VueApexCharts);
app.use(VueGtag, {
  property: {
    id: "G-W064G2CBKJ"
  }
});
app.mount('#app')
