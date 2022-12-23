<template>
  <h3>
    <router-link to="/dashboard">
      Apps
    </router-link>
    -
    {{ appname }}
  </h3>
  <div v-if="app" class="appinfo">
    <div class="status">
      Status:
      <span :class="app.status">{{ app.status }}</span>
    </div>
    <div>
      Storage usage: {{ app.storage_usage_mb / app.storage_limit_mb }}%
      ({{ app.storage_usage_mb }} / {{ app.storage_limit_mb }} MiB)
    </div>
    <div class="chart">
      <span>
        API query count per
        <select class="unit" v-model="query_unit" @change="reload_query">
          <option value="minute">minute</option>
          <option value="hour">hour</option>
          <option value="day">day</option>
        </select>
      </span>
      <apexchart width="100%" height="200px" type="area" :options="chart_options" :series="chart_series">
      </apexchart>
    </div>
    <div>
      App admin token:
      <span class="admin-token" v-if="show_admin_token">
        {{ app.info.admin_token }}
      </span>
      <button class="btn btn-secondary" v-if="show_admin_token" @click="show_admin_token=false">Hide</button>
      <button class="btn btn-secondary" v-else @click="show_admin_token=true">Show</button>
    </div>
    <div>
      Base URI: {{ app_url }}
    </div>
    <div>
      Status API: <a target="_blank" :href="status_url">{{ status_url }}</a>
    </div>
    <div>
      <label>Git:</label>
      <input class="form-control" type="text" v-model="git_repo"/>
    </div>
    <div>
      <label>Head ref (branch/tag/commit):</label>
      <input class="form-control" type="text" v-model="git_ref"/>
    </div>
    <div>
      <button class="btn btn-primary" @click="update">Update</button>
    </div>
    <div class="events">
      <h4>
        Events
      </h4>
      <div class="entries">
        <ul v-if="app.events.length > 0">
          <li v-for="(evt, i) in app.events" :key="i">
            <span class="time">
              [{{ evt.datetime }}]
            </span>
            {{ evt.content }}
          </li>
        </ul>
        <i v-else>
          // No events
        </i>
      </div>
    </div>
    <div class="management">
      <h4>Management</h4>
      <div>
        <button class="btn btn-danger" @click="del">Delete App</button>
      </div>
    </div>
  </div>
</template>

<script>

export default {
  name: 'AppOverview',
  props: ['appname'],
  components: {
  },
  data: function () {
    return {
      app: null,
      show_admin_token: false,
      git_repo: null,
      git_ref: null,
      query_unit: "minute",
      query_data: [],
    };
  },
  computed: {
    app_url: function() {
      var rel = this.app.base_uri;
      return new URL(rel, document.baseURI).href;
    },
    status_url: function () {
      return this.app_url + "/__oct_status";
    },
    query_count_url: function () {
      return this.app_url + "/__oct_query_count?unit=" + this.query_unit;
    },
    chart_series: function() {
      return [{
      name: 'Queries',
      data: this.query_data,
      }];
    },
    chart_options: function() {
      var cat = [];
      for (var i = this.query_data.length - 1; i > 0; i--) {
        var s = "";
        if (i > 1) {
          s = "s";
        }
        cat.push(i.toString() + " " + this.query_unit + s + " ago");
      }
      cat.push("now");

      return {
        theme: {
          mode: 'dark',
        },
        chart: {
          animations: { enabled: false, },
          toolbar: {
            show: false,
          },
          zoom: {
            enabled: false,
          },
        },
        dataLabels: {
          enabled: false,
        },
        xaxis:{
          labels: {
            show: false,
          },
          categories: cat,
        },
      };
    },
  },
  methods: {
    reload_query: async function() {
      var r = await this.axios.get(this.query_count_url);
      this.query_data = r.data.reverse();
    },
    reload_info: async function() {
      var r = await this.$root.api_get("/meta/app");
      for (var a of r) {
        if (a.info.name == this.appname) {
          this.app = a;
          if (!this.git_repo) this.git_repo = a.info.git_repo;
          if (!this.git_ref) this.git_ref = a.info.git_ref;
          break;
        }
      }
    },
    reload: async function() {
      await this.reload_info();
      this.reload_query();
    },
    save: async function() {
      this.app.info.git_repo = this.git_repo;
      this.app.info.git_ref = this.git_ref;
      await this.$root.api_put("/meta/app", this.app.info);
    },
    update: async function() {
      await this.save();
      await this.$root.api_post("/meta/sync", { name: this.app.info.name } );
      this.reload();
    },
    del: async function() {
      if (confirm("Are you sure about deleting the app and all the data of " + this.appname)) {
        await this.$root.api_delete("/meta/app", { name: this.app.info.name } );
        this.$router.push("/dashboard");
      }
    },
  },
  mounted() {
    this.reload();
    setInterval(this.reload, 10000);
  },
}
</script>

<style scoped>
div.events {
  margin-top: 2rem;
}
div.entries {
  border: 1px dashed #999;
  padding: 1rem 1rem 1rem 2rem;
  max-height: 300px;
  overflow-y: scroll;
  background: #222;
}

input {
  width: 100%;
  box-sizing: border-box;
}

div.chart {
  text-align: center;
  border: 1px dotted #888;
  padding: 0.5rem;
}

div.chart span {
  display: block;
  margin-bottom: 1rem;
}
span.admin-token {
  margin-left: 1rem;
  margin-right: 1rem;
}
select.unit {
  display: inline-block;
}
span.time {
  color: #888;
}
div.status > span {
  font-weight: bold;
}
div.status > .PENDING {
  color: #fa0;
}
div.status > .RUNNING {
  color: #0f0;
}
div.appinfo > div {
  margin: 1rem 0;
}
</style>

