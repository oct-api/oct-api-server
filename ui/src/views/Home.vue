<template>
  <div class="banner">
    <h2 class="jumbo">
      <strong>
        Building APIs
      </strong>
        now made easy,<br>
        with
      <strong>
        less code
      </strong>
        and
      <strong>
        more power
      </strong>
    </h2>
    <p class="mt-4 small">
    Oct API a <i>low-code</i> and <i>serverless</i> API builder service.
    </p>
    <p>
      Start an API service in a few minutes!
      Take a look at
      <a href="/doc/" target="_blank">the tutorial</a>.
    </p>
    <button class="btn btn-light mt-4" @click="$root.login">
      <img src="https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png" height="26" width="26" alt=""/>
      Login with GitHub
    </button>
  </div>
  <hr class="w-75 mx-auto">
  <div class="intro">
    <div>
      <h4>Truly for <strong>developers</strong></h4>
      <p>
        Oct API takes a different approach to low-code.
      </p>
      <p>
        We do everything possible to remove the boilerplate in building a
        backend service, but we don't try to sell software development to those
        who don't know how to write code. There is no drag-and-drop, everything
        is <strong>version controlled</strong> in git.
      </p>
    </div>
    <div>
      <h4>It's <strong>free</strong>!</h4>
      <p>You have access to 1 GiB of storage, and it's always free.</p>
      <p>Simply sign-in with GitHub OAuth, no registration required.</p>
    </div>
    <div>
      <h4><strong>Low-code</strong>, but don't no-code</h4>
      <p>
        Define your data models, API methods and authorization policies with a
        high level YAML syntax.
      </p>
      <p>
        Use git to manage your app definition, resources and code snippets;
        integrate with existing CI/CD workflows easily.
      </p>
    </div>
    <div>
      <h4>Batteries included</h4>
      <p>
        With built-in <strong>auth models, permission control</strong> and
        necessary <strong>API handlers</strong>, you can enable multi-tenant usage easily.  </p>
      <p>
        Get insights into your service with built-in <strong>monitoring</strong>.
      </p>
    </div>
    <div class="start-now">
      <a href="/doc/" target="_blank" class="btn btn-secondary mx-auto px-4">Read more</a>
    </div>
  </div>
</template>

<script>

export default {
  name: 'Home',
  components: {
  }
}
</script>

<style>
.banner {
  margin: 8rem 0 8rem 0;
  display: block;
}
.banner p.small {
  color: #aaa;
}
div.intro {
  max-width: 800px;
}
.intro strong {
  color: #FFB703;
  font-weight: normal;
}

.intro > div {
  margin-top: 3rem;
}

.intro h4 {
  margin-bottom: 1.5rem;
}
div.start-now a {
  margin: 5rem 0;
  display: block;
  max-width: 300px;
}
</style>
