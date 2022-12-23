<template>
  login confirmed
</template>

<script>
export default {
  name: '',
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
    var token = this.$route.params.token;
    localStorage.setItem('token', token);
    this.$root.$data.token = token;
    this.$root.init();
    this.$router.push('/');
  },
}
</script>

<style scoped>
</style>

