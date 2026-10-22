# Rustic Player — Frozen Engine API Contract

**API Version:** `1.0.0`  
**Protocol Version:** `1`  
**Milestone Gate:** S0 (Frozen)

This document contains the frozen Command & Event contract schemas that govern all communication between Track A (Core Engine) and Track B (UI).

---

## 1. Command Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Command",
  "type": "object",
  "required": [
    "action"
  ],
  "properties": {
    "action": {
      "$ref": "#/definitions/CommandAction"
    },
    "origin": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "CommandAction": {
      "oneOf": [
        {
          "type": "string",
          "enum": [
            "Play",
            "Pause",
            "TogglePlay",
            "Stop",
            "Next",
            "Previous",
            "RescanChanged"
          ]
        },
        {
          "type": "object",
          "required": [
            "Seek"
          ],
          "properties": {
            "Seek": {
              "$ref": "#/definitions/Duration"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "SetVolume"
          ],
          "properties": {
            "SetVolume": {
              "type": "number",
              "format": "float"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "ResolveAndPlay"
          ],
          "properties": {
            "ResolveAndPlay": {
              "type": "string"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "Login"
          ],
          "properties": {
            "Login": {
              "type": "object",
              "required": [
                "provider"
              ],
              "properties": {
                "provider": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "ImportPlaylist"
          ],
          "properties": {
            "ImportPlaylist": {
              "type": "object",
              "required": [
                "external_id",
                "provider"
              ],
              "properties": {
                "external_id": {
                  "type": "string"
                },
                "provider": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "PlayTrack"
          ],
          "properties": {
            "PlayTrack": {
              "type": "string"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "PlayContext"
          ],
          "properties": {
            "PlayContext": {
              "type": "object",
              "required": [
                "context",
                "start_at"
              ],
              "properties": {
                "context": {
                  "$ref": "#/definitions/PlayContext"
                },
                "start_at": {
                  "type": "integer",
                  "format": "uint",
                  "minimum": 0.0
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "EnqueueNext"
          ],
          "properties": {
            "EnqueueNext": {
              "type": "array",
              "items": {
                "type": "string"
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "EnqueueEnd"
          ],
          "properties": {
            "EnqueueEnd": {
              "type": "array",
              "items": {
                "type": "string"
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "RemoveFromQueue"
          ],
          "properties": {
            "RemoveFromQueue": {
              "type": "integer",
              "format": "uint",
              "minimum": 0.0
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "MoveInQueue"
          ],
          "properties": {
            "MoveInQueue": {
              "type": "object",
              "required": [
                "from",
                "to"
              ],
              "properties": {
                "from": {
                  "type": "integer",
                  "format": "uint",
                  "minimum": 0.0
                },
                "to": {
                  "type": "integer",
                  "format": "uint",
                  "minimum": 0.0
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "SetShuffle"
          ],
          "properties": {
            "SetShuffle": {
              "type": "boolean"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "SetRepeat"
          ],
          "properties": {
            "SetRepeat": {
              "$ref": "#/definitions/RepeatMode"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "ScanFolders"
          ],
          "properties": {
            "ScanFolders": {
              "type": "array",
              "items": {
                "type": "string"
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "Search"
          ],
          "properties": {
            "Search": {
              "type": "object",
              "required": [
                "limit",
                "query"
              ],
              "properties": {
                "limit": {
                  "type": "integer",
                  "format": "uint",
                  "minimum": 0.0
                },
                "query": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        }
      ]
    },
    "Duration": {
      "type": "object",
      "required": [
        "nanos",
        "secs"
      ],
      "properties": {
        "nanos": {
          "type": "integer",
          "format": "uint32",
          "minimum": 0.0
        },
        "secs": {
          "type": "integer",
          "format": "uint64",
          "minimum": 0.0
        }
      }
    },
    "PlayContext": {
      "oneOf": [
        {
          "type": "string",
          "enum": [
            "AllTracks"
          ]
        },
        {
          "type": "object",
          "required": [
            "Album"
          ],
          "properties": {
            "Album": {
              "type": "string"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "Artist"
          ],
          "properties": {
            "Artist": {
              "type": "string"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "Playlist"
          ],
          "properties": {
            "Playlist": {
              "type": "string"
            }
          },
          "additionalProperties": false
        }
      ]
    },
    "RepeatMode": {
      "type": "string",
      "enum": [
        "Off",
        "All",
        "One"
      ]
    }
  }
}
```

---

## 2. Event Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Event",
  "oneOf": [
    {
      "type": "string",
      "enum": [
        "LibraryChanged"
      ]
    },
    {
      "type": "object",
      "required": [
        "PlaybackStateChanged"
      ],
      "properties": {
        "PlaybackStateChanged": {
          "$ref": "#/definitions/PlaybackState"
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "PositionChanged"
      ],
      "properties": {
        "PositionChanged": {
          "type": "object",
          "required": [
            "duration_secs",
            "position_secs"
          ],
          "properties": {
            "duration_secs": {
              "type": "number",
              "format": "double"
            },
            "position_secs": {
              "type": "number",
              "format": "double"
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "TrackChanged"
      ],
      "properties": {
        "TrackChanged": {
          "anyOf": [
            {
              "$ref": "#/definitions/Track"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "Resolved"
      ],
      "properties": {
        "Resolved": {
          "type": "object",
          "required": [
            "audio",
            "track_id"
          ],
          "properties": {
            "audio": {
              "$ref": "#/definitions/ResolvedAudio"
            },
            "track_id": {
              "type": "string"
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "VolumeChanged"
      ],
      "properties": {
        "VolumeChanged": {
          "type": "number",
          "format": "float"
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "QueueChanged"
      ],
      "properties": {
        "QueueChanged": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/Track"
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "ShuffleChanged"
      ],
      "properties": {
        "ShuffleChanged": {
          "type": "boolean"
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "RepeatChanged"
      ],
      "properties": {
        "RepeatChanged": {
          "$ref": "#/definitions/RepeatMode"
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "AuthStateChanged"
      ],
      "properties": {
        "AuthStateChanged": {
          "type": "object",
          "required": [
            "logged_in",
            "provider"
          ],
          "properties": {
            "logged_in": {
              "type": "boolean"
            },
            "provider": {
              "type": "string"
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "LyricsChanged"
      ],
      "properties": {
        "LyricsChanged": {
          "type": "object",
          "required": [
            "track_id"
          ],
          "properties": {
            "lrc_content": {
              "type": [
                "string",
                "null"
              ]
            },
            "track_id": {
              "type": "string"
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "JamSessionChanged"
      ],
      "properties": {
        "JamSessionChanged": {
          "type": "object",
          "required": [
            "peers"
          ],
          "properties": {
            "peers": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "session_id": {
              "type": [
                "string",
                "null"
              ]
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "ScanProgress"
      ],
      "properties": {
        "ScanProgress": {
          "type": "object",
          "required": [
            "done",
            "total"
          ],
          "properties": {
            "done": {
              "type": "integer",
              "format": "uint",
              "minimum": 0.0
            },
            "total": {
              "type": "integer",
              "format": "uint",
              "minimum": 0.0
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "ScanComplete"
      ],
      "properties": {
        "ScanComplete": {
          "type": "object",
          "required": [
            "added",
            "removed",
            "updated"
          ],
          "properties": {
            "added": {
              "type": "integer",
              "format": "uint",
              "minimum": 0.0
            },
            "removed": {
              "type": "integer",
              "format": "uint",
              "minimum": 0.0
            },
            "updated": {
              "type": "integer",
              "format": "uint",
              "minimum": 0.0
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "SearchResults"
      ],
      "properties": {
        "SearchResults": {
          "type": "object",
          "required": [
            "query",
            "results"
          ],
          "properties": {
            "query": {
              "type": "string"
            },
            "results": {
              "$ref": "#/definitions/SearchResults"
            }
          }
        }
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "Error"
      ],
      "properties": {
        "Error": {
          "$ref": "#/definitions/EngineError"
        }
      },
      "additionalProperties": false
    }
  ],
  "definitions": {
    "EngineError": {
      "type": "object",
      "required": [
        "kind",
        "message"
      ],
      "properties": {
        "kind": {
          "$ref": "#/definitions/ErrorKind"
        },
        "message": {
          "type": "string"
        }
      }
    },
    "ErrorKind": {
      "type": "string",
      "enum": [
        "Network",
        "Auth",
        "Resolve",
        "Decode",
        "Library",
        "Internal"
      ]
    },
    "PlaybackState": {
      "type": "string",
      "enum": [
        "Stopped",
        "Playing",
        "Paused",
        "Buffering"
      ]
    },
    "RepeatMode": {
      "type": "string",
      "enum": [
        "Off",
        "All",
        "One"
      ]
    },
    "ResolvedAudio": {
      "type": "object",
      "required": [
        "bitrate_kbps",
        "codec",
        "stream_url"
      ],
      "properties": {
        "bitrate_kbps": {
          "type": "integer",
          "format": "uint32",
          "minimum": 0.0
        },
        "codec": {
          "type": "string"
        },
        "expires_at_unix_secs": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint64",
          "minimum": 0.0
        },
        "stream_url": {
          "type": "string"
        }
      }
    },
    "SearchResults": {
      "type": "object",
      "required": [
        "tracks"
      ],
      "properties": {
        "tracks": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/Track"
          }
        }
      }
    },
    "Track": {
      "type": "object",
      "required": [
        "album",
        "artists",
        "duration_secs",
        "id",
        "source",
        "title"
      ],
      "properties": {
        "album": {
          "type": "string"
        },
        "artists": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "artwork_url": {
          "type": [
            "string",
            "null"
          ]
        },
        "content_hash": {
          "type": [
            "string",
            "null"
          ]
        },
        "duration_secs": {
          "type": "integer",
          "format": "uint32",
          "minimum": 0.0
        },
        "id": {
          "type": "string"
        },
        "isrc": {
          "type": [
            "string",
            "null"
          ]
        },
        "source": {
          "$ref": "#/definitions/TrackSource"
        },
        "title": {
          "type": "string"
        }
      }
    },
    "TrackSource": {
      "oneOf": [
        {
          "type": "object",
          "required": [
            "Local"
          ],
          "properties": {
            "Local": {
              "type": "string"
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "YouTubeMusic"
          ],
          "properties": {
            "YouTubeMusic": {
              "type": "object",
              "required": [
                "video_id"
              ],
              "properties": {
                "video_id": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": [
            "Spotify"
          ],
          "properties": {
            "Spotify": {
              "type": "object",
              "required": [
                "spotify_id"
              ],
              "properties": {
                "spotify_id": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        }
      ]
    }
  }
}
```
