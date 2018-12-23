# Think of a cooler name

This component allows the outside to interact with the cluster. The client connection is persistent, meaning they have to re-authenticate if the socket gets disconnected. This node doesn't store any long-lived state so can be scaled. This exposes an API to allow the client to do different things like:
 - character creation/selection/maintenance
 - account settings
 - creation of the relay