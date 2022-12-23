<template>
  <h3>
    <router-link to="/dashboard">
      Dashboard
    </router-link>
  </h3>
  <div class="apps my-3" v-if="apps">
    <h4>
      My apps:
    </h4>
    <div class="app" v-for="(app, i) in apps" :key="i">
      <router-link class="applink" :to="{name: 'AppOverview', params: { appname: app.info.name }}">
        {{ app.info.name }}
      </router-link>
    </div>
    <div class="mt-3">
      <label for="newappname" class="form-label">Create a new app:</label>
      <input id="newappname" class="form-control" type="text" placeholder="new app name" v-model="new_app_name"/>
      <button class="mt-2 btn btn-primary" @click="create">Create</button>
    </div>
  </div>
  <div class="boxed">
    <h4>Admin token:</h4>
    <p><i>
      You can interact with <strong>oct-api.com</strong> with your admin token programmably.
      <a href="/doc/" target="_blank">Read more about it.</a>
    </i>
    </p>
    <div>
      <button class="btn btn-secondary" v-if="!show_token" @click="show_token=true">Show</button>
      <button class="btn btn-secondary" v-else @click="show_token=false">Hide</button>
      <div class="mt-3">
        <span class="token" v-if="show_token">
          {{ $root.token }}
        </span>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Dashboard',
  props: [],
  components: {
  },
  data: function () {
    return {
      apps: [],
      new_app_name: '',
      show_token: false,
    };
  },
  computed: {
  },
  methods: {
    reload: async function () {
      var r = await this.$root.api_get("/meta/app");
      this.apps = r;
    },
    create: async function() {
      if (!this.new_app_name)
        return;
      var data = {
        name: this.new_app_name,
      };
      await this.$root.api_post('/meta/app', data);
      this.$router.push('/app/' + this.new_app_name);
    }
  },
  mounted() {
    this.reload();
  },
}
</script>

<style scoped>

div.apps label {
  margin-right: 2rem;
}

div.boxed {
  border: 2px dotted;
  border-color: rgba(255, 167, 21, 0.5);
  border-radius: 5px;
  padding: 1rem;
}

div.create label {
  margin-bottom: .5rem;
}

div.app {
  margin: 1rem 1rem 1rem 0;
  display: inline-block;
  padding: 0.5rem 1rem;
  border: 2px solid;
  border-color: rgba(255, 167, 21, 0.8);
  border-radius: 15px;
  background-color: rgba(0, 0, 0, 0.3);
}

div.app:hover {
  border-color: rgba(255, 167, 21, 1);
}

span.token {
  margin-right: 1rem;
}
</style>

