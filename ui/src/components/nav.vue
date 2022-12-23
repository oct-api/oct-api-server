<template>
  <nav>
    <router-link class="title" to="/">
      <img src="/logo.png" height="24" width="24" alt=""/>
      Oct-API
      <span class="beta rounded-pill badge bg-success">Beta</span>
    </router-link>
    <span class="pull-right">
      <router-link v-if="$root.myself" to="/dashboard">Dashboard</router-link>
      <a href="/doc/" target="_blank">Docs</a>
      <span class="user">
        <span v-if="$root.myself">
          Hello, <strong>{{ $root.myself.display_name }}</strong>
        </span>
        <button v-if="!$root.token" class="btn btn-light mt-4" @click="$root.login">
          <img src="https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png" height="26" width="26" alt=""/>
          Sign-in
        </button>
      </span>
    </span>
  </nav>
</template>

<script>
export default {
  name: 'Nav',
  props: [],
  components: {
  },
  data: function () {
    return {
    };
  },
  computed: {
  },
  methods: {
  },
  mounted() {
  },
}
</script>

<style scoped>

nav {
  font-size: 1.1rem;
  border-bottom: dashed #8d8c8a 1px;
  padding-bottom: 0.3rem;
}

a {
  margin-right: 1.5rem;
}

button {
  margin: 1rem;
}

.title {
  padding: 0 0 0.3rem 0;
  font-weight: bold;
  font-size: 26px;
  margin-right: 3rem;
  color: #52d6f4;
}

.title:hover {
  text-decoration: none;
}

.title img {
  display: inline-block;
  margin-bottom: 0.3rem;
}

span.beta {
  font-size: 35%;
}

span.octagon {
  color: #ffa715;
}

</style>

