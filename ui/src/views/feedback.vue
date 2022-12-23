<template>
  <div>
    <h3 class="mb-4">User feedback</h3>
    <div>
      <p>
        Hello!
      </p>
      <p>
        Thank you so much for helping out Oct API! Please leave your email and
        comments below, and we will get back to you ASAP.
      </p>
    </div>
    <label class="form-label" for="email">Email:</label>
    <input class="form-control" id="email" type="text" v-model="email" />
    <label class="form-label mt-3" for="comments">Comments:</label>
    <textarea class="form-control" v-model="comments" rows="10"></textarea>
    <div v-if="result" class="my-3 alert alert-success">
      {{ result }}
    </div>
    <button @click="submit" class="btn btn-primary my-3">Submit</button>
  </div>
</template>

<script>
export default {
  name: 'Feedback',
  props: [],
  components: {
  },
  data: function () {
    return {
      email: "",
      comments: "",
      result: "",
    };
  },
  computed: {
  },
  methods: {
    do_submit: async function() {
      var data = {
        email: this.email,
        comments: this.comments,
      };
      var r = await this.axios.post("/feedback/", data);
      if (r.data == true) {
        this.email = "";
        this.comments = "";
        this.result = "Your feedback is submitted, thank you!";
      }
    },
    submit: function() {
      if (this.email != "" && this.comments != "") {
        this.do_submit();
      }
    },
  },
  mounted() {
  },
}
</script>

<style scoped>
</style>

