# Generated by Django 3.0.5 on 2021-12-02 17:42

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('rest', '0001_initial'),
    ]

    operations = [
        migrations.AlterField(
            model_name='user',
            name='token',
            field=models.CharField(db_index=True, max_length=100),
        ),
    ]