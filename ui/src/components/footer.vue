<template>
  <div class="footer">
    <hr>
    <div class="row">
      <div class="col-lg-6">
        <div>
          <router-link :to="{name:'Feedback'}">
            Leave comments
          </router-link>
        </div>
        <div>
          <router-link :to="{name:'Feedback'}">
            Report an issue
          </router-link>
        </div>
        <div>
          <a href="https://www.reddit.com/r/OctAPI/" target="_blank">Reddit /r/OctAPI</a>
        </div>
        <div>
          <a href="https://theoctapiblog.wordpress.com/" target="_blank">Dev Blog</a>
        </div>
        <div>
          Email:
          <a href="mailto:contact.octapi@gmail.com" target="_blank">
            contact.octapi@gmail.com
          </a>
        </div>
      </div>
    </div>
    <div class="copyright">
      Copyright 2021-2022, oct-api.com.  All rights reserved
    </div>
  </div>
</template>

<script>
export default {
  name: 'Footer',
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
div.footer {
  color: #888;
  margin-top: 10rem;
}
div.copyright {
  margin-top: 2rem;
}
a {
  color: #777;
}
a:hover {
  color: #999;
}
div.row {
  margin-top: 3rem;
}
</style>

