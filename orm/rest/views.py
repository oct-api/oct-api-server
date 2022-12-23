from rest_framework import routers, serializers, viewsets
from django_filters.rest_framework import DjangoFilterBackend
from .models import *

class BaseViewSets(viewsets.ModelViewSet):
    filter_backends = [DjangoFilterBackend]
    filterset_fields = "__all__"

class UserViewSet(BaseViewSets):
    class ViewSetSerializer(serializers.ModelSerializer):
        class Meta:
            model = User
            fields = "__all__"
    name = 'user'

    queryset = ViewSetSerializer.Meta.model.objects.all()
    serializer_class = ViewSetSerializer

class AppViewSet(BaseViewSets):
    class ViewSetSerializer(serializers.ModelSerializer):
        class Meta:
            model = App
            fields = "__all__"
    name = 'app'

    queryset = ViewSetSerializer.Meta.model.objects.all()
    serializer_class = ViewSetSerializer

class AppEventViewSet(BaseViewSets):
    class ViewSetSerializer(serializers.ModelSerializer):
        class Meta:
            model = AppEvent
            fields = "__all__"
    name = 'event'

    queryset = ViewSetSerializer.Meta.model.objects.all().order_by('-datetime')
    serializer_class = ViewSetSerializer

router = routers.DefaultRouter()
for cls in BaseViewSets.__subclasses__():
    router.register(cls.name, cls)
