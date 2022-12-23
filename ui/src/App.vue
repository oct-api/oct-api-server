<template>
  <div>
    <Nav />
    <router-view />
  </div>
  <Footer />
</template>

<script>
import Nav from './components/nav.vue';
import Footer from './components/footer.vue';
export default {
  name: 'App',
  props: [],
  components: {
    Nav,
    Footer,
  },
  data: function () {
    return {
      token: null,
      myself: null,
    };
  },
  computed: {
  },
  methods: {
    login: async function() {
      var r = await this.axios.get("/auth/github/config");
      var cfg = r.data;
      var query = "client_id=" + cfg.client_id;
      query += "&redirect_uri=" + cfg.redirect_uri_base + "/auth/github/callback";
      var url = 'https://github.com/login/oauth/authorize?' + query;
      window.location.href = url;
    },
    logout: function() {
      localStorage.removeItem('token');
      this.token = null;
      this.myself = null;
    },
    api_get: async function(path) {
      var r = await this.axios.get(path, {
          headers: {
            "Authorization": "token " + this.token,
          }
        }
      );
      return r.data;
    },
    api_post: async function(path, data) {
      var r = await this.axios.post(path, data,
        {
          headers: {
            "Authorization": "token " + this.token,
          }
        }
      );
      return r.data;
    },
    api_put: async function(path, data) {
      var r = await this.axios.put(path, data,
        {
          headers: {
            "Authorization": "token " + this.token,
          }
        }
      );
      return r.data;
    },
    api_delete: async function(path, data) {
      var r = await this.axios.delete(path, {
        headers: {
          "Authorization": "token " + this.token,
        },
        data,
      });
      return r.data;
    },
    init: async function() {
      this.token = localStorage.getItem('token');
      if (this.token) {
        var r = await this.api_get("/auth/info");
        this.myself = r;
      }
    },
  },
  mounted() {
    this.init();
  },
}
</script>

<style>
/* https://coolors.co/8ecae6-219ebc-023047-ffb703-fb8500 */
body {
  padding: 3rem;
  color: #f6f6f6;
  background-color: #023047;
  background: linear-gradient(90deg, rgba(32,32,42,1) 0%, rgba(2,35,51,1) 100%);
}

a {
  text-decoration: none;
  color: #8ECAE6;
}

a:hover {
  color: #FFB703;
}

nav {
  margin-bottom: 1rem;
}

</style>

