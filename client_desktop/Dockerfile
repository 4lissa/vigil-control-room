FROM alpine:3.20
RUN apk add --no-cache curl
ARG GITHUB_REPO=4lissa/vigil-control-room
ENV GITHUB_REPO=${GITHUB_REPO}
CMD curl -fsSL -o /shared/client.dmg \
  "https://github.com/${GITHUB_REPO}/releases/latest/download/vigil.dmg" \
  || echo "no release published yet, skipping desktop binary delivery"
