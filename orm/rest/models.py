from django.db import models

class User(models.Model):
    username = models.CharField(max_length=100, unique=True, db_index=True)
    email = models.CharField(max_length=100, null=True, blank=True)
    token = models.CharField(max_length=100, db_index=True)
    display_name = models.CharField(max_length=100)

class App(models.Model):
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    name = models.SlugField()
    handle = models.CharField(max_length=100, unique=True)
    admin_token = models.CharField(max_length=100)
    git_repo = models.TextField(null=True, blank=True)
    git_ref = models.TextField(null=True, blank=True)
    yml = models.TextField(null=True, blank=True)

    class Meta:
        unique_together = ('user', 'name')

class AppEvent(models.Model):
    app = models.ForeignKey(App, on_delete=models.CASCADE)
    datetime = models.DateTimeField(auto_now=True)
    content = models.TextField()
