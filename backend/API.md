

 ## Yamba-Daemon API

REST API for yamba-daemon 

 ### Instance
 
 - GET `/instance/list` returns `InstanceListResponse` with ID & startup time
 - POST `/instance/stop` with body `InstanceStopReq`  stops instance
 - POST `/instance/start` with body `InstanceLoadReq`  stops instance
 - GET `/instance/state` with query params `StateGetReq` returns current `InstanceStateResponse` for instance
##### Callbacks
- POST `PATH_INSTANCE` with `InstanceStateResponse` on instance state change

#### Resolve
- GET `/resolve/url` with query params `ResolveRequest` returns `ResolveTicketResponse` on success, see callbacks
##### Callbacks
- POST `PATH_RESOLVE` with `ResolveResponse` on URL resolve finish

#### Playback
- POST `/playback/url` with body `PlaybackUrlReq` starts playback with specified track
- POST `/playback/pause` with body `PlaybackPauseReq` toggle pause for current playback
- GET `/playback/state` with query params `StateGetReq` returns `PlaystateResponse`
- POST `/volume` with body `VolumeSetReq` sets volume
- GET `/volume` with query params `VolumeGetReq` returns `VolumeResponse`
##### Callbacks
- POST `PATH_PLAYBACK` with `PlaystateResponse` on playback change
~~- POST `PATH_SONG` with `` on song metadata change~~
- POST `PATH_VOLUME` with `VolumeChange` on volume change
- POST `PATH_POSITION` with `TrackPositionUpdate` on position change

Copyright :copyright: Aron Heinecke 2019
