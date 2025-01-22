docker build -t sashvazquez/bitcoin-node:latest .
docker push sashvazquez/bitcoin-node:latest

kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml

# kubectl get pods
# kubectl get svc

kubectl scale deployment bitcoin-node --replicas=15

# kubectl logs -l app=bitcoin --tail=100 -f