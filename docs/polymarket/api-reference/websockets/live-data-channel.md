> ## Documentation Index
> Fetch the complete documentation index at: https://docs.polymarket.com/llms.txt
> Use this file to discover all available pages before exploring further.

# Live Data Channel

> Send JSON text frames tagged with op. The server answers every op with an ack or an error ack (never a close, except for policy breaches, close 4008) and pushes envelopes for subscribed channels. Limits per connection: 64 active (channel, filter) subscriptions, 20 subscribe or unsubscribe frames per second (a batch frame counts once), 64 KB per frame, 8 auth attempts. The server sends a WebSocket ping every 25 seconds; two missed pongs close 4002. Close codes: 4001 authentication failed on a presented credential, 4002 slow consumer or pong timeout, 4003 server draining (reconnect once after a uniform 0 to 10 s delay), 4008 policy violation (fix the client; do not reconnect blindly).



## AsyncAPI

````yaml https://ws-live-v2.polymarket.com/asyncapi.json ws
id: ws
title: Live Data Channel
description: >-
  Send JSON text frames tagged with op. The server answers every op with an ack
  or an error ack (never a close, except for policy breaches, close 4008) and
  pushes envelopes for subscribed channels. Limits per connection: 64 active
  (channel, filter) subscriptions, 20 subscribe or unsubscribe frames per second
  (a batch frame counts once), 64 KB per frame, 8 auth attempts. The server
  sends a WebSocket ping every 25 seconds; two missed pongs close 4002. Close
  codes: 4001 authentication failed on a presented credential, 4002 slow
  consumer or pong timeout, 4003 server draining (reconnect once after a uniform
  0 to 10 s delay), 4008 policy violation (fix the client; do not reconnect
  blindly).
servers:
  - id: production
    protocol: wss
    host: ws-live-v2.polymarket.com
    bindings: []
    variables: []
address: /ws
parameters: []
bindings: []
operations:
  - &ref_7
    id: subscribe
    title: Subscribe
    description: Add one or more (channel, filter) subscriptions
    type: receive
    messages:
      - &ref_18
        id: subscribe
        payload:
          - oneOf: &ref_0
              - type: object
                required:
                  - op
                  - subscriptions
                properties:
                  op:
                    const: subscribe
                    x-parser-schema-id: <anonymous-schema-2>
                  subscriptions:
                    type: array
                    minItems: 1
                    items: &ref_1
                      type: object
                      required:
                        - channel
                        - filter
                      properties:
                        channel:
                          type: string
                          enum:
                            - price.polymarket
                            - price.crypto
                            - price.equity
                            - price.crypto.twap
                            - price.equity.twap
                          x-parser-schema-id: <anonymous-schema-4>
                        filter:
                          oneOf:
                            - type: object
                              required:
                                - asset_id
                              properties:
                                asset_id:
                                  type: string
                                  pattern: ^[0-9]{1,78}$
                                  description: >-
                                    Outcome token id (clobTokenIds entry),
                                    decimal string. Leading zeros are
                                    normalized.
                                  x-parser-schema-id: <anonymous-schema-6>
                              x-parser-schema-id: AssetIdFilter
                            - type: object
                              required:
                                - symbol
                              properties:
                                symbol:
                                  type: string
                                  description: >-
                                    Lowercase symbol. Matching is
                                    case-insensitive.
                                  x-parser-schema-id: <anonymous-schema-7>
                                provider:
                                  type: string
                                  enum:
                                    - chainlink
                                    - pyth
                                  description: >-
                                    Optional provider pin (spec 08b). While the
                                    server's provider selector is DISABLED
                                    (PRICE_PROVIDER_SELECTOR_ENABLED=false, the
                                    default), this field is accepted and IGNORED
                                    like any unknown key: the lane's owner is
                                    served and the subscribed ack carries no
                                    provider. While it is ENABLED, the value
                                    must be one of the enum tokens; anything
                                    else (including massive, owner, numbers,
                                    null) is a bad_filter error ack. A pin the
                                    catalog serves gets that provider's stream;
                                    a pin it cannot serve falls back to the
                                    owner's stream, and the subscribed ack names
                                    the provider actually served. Ignored on
                                    price.polymarket.
                                  x-parser-schema-id: <anonymous-schema-8>
                              x-parser-schema-id: SymbolFilter
                            - type: object
                              required:
                                - symbol
                                - window_seconds
                              properties:
                                symbol:
                                  type: string
                                  x-parser-schema-id: <anonymous-schema-9>
                                window_seconds:
                                  type: integer
                                  minimum: 1
                                  description: >-
                                    Only 60 carries data today, on
                                    price.crypto.twap and price.equity.twap
                                    alike.
                                  x-parser-schema-id: <anonymous-schema-10>
                                provider:
                                  type: string
                                  enum:
                                    - chainlink
                                    - pyth
                                  description: >-
                                    Optional provider pin (spec 08b). While the
                                    server's provider selector is DISABLED
                                    (PRICE_PROVIDER_SELECTOR_ENABLED=false, the
                                    default), this field is accepted and IGNORED
                                    like any unknown key: the lane's owner is
                                    served and the subscribed ack carries no
                                    provider. While it is ENABLED, the value
                                    must be one of the enum tokens; anything
                                    else (including massive, owner, numbers,
                                    null) is a bad_filter error ack. A pin the
                                    catalog serves gets that provider's stream;
                                    a pin it cannot serve falls back to the
                                    owner's stream, and the subscribed ack names
                                    the provider actually served. Ignored on
                                    price.polymarket.
                                  x-parser-schema-id: <anonymous-schema-11>
                              x-parser-schema-id: TwapFilter
                          x-parser-schema-id: <anonymous-schema-5>
                      x-parser-schema-id: SubscriptionItem
                    x-parser-schema-id: <anonymous-schema-3>
                  rid:
                    type: string
                    description: >-
                      Client request id, echoed on every ack this frame
                      produces.
                    x-parser-schema-id: <anonymous-schema-12>
                x-parser-schema-id: <anonymous-schema-1>
              - type: object
                required:
                  - op
                  - channel
                  - filter
                properties:
                  op:
                    const: subscribe
                    x-parser-schema-id: <anonymous-schema-14>
                  channel:
                    type: string
                    x-parser-schema-id: <anonymous-schema-15>
                  filter:
                    type: object
                    x-parser-schema-id: <anonymous-schema-16>
                  rid:
                    type: string
                    x-parser-schema-id: <anonymous-schema-17>
                x-parser-schema-id: <anonymous-schema-13>
            x-parser-schema-id: SubscribeRequest
            name: subscribe
        headers: []
        jsonPayloadSchema:
          oneOf: *ref_0
          x-parser-schema-id: SubscribeRequest
        title: Subscribe
        example: |-
          {
            "op": "subscribe",
            "rid": "s1",
            "subscriptions": [
              {
                "channel": "price.polymarket",
                "filter": {
                  "asset_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455"
                }
              },
              {
                "channel": "price.crypto",
                "filter": {
                  "symbol": "btcusd"
                }
              }
            ]
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: subscribe
    bindings: []
    extensions: &ref_3
      - id: x-parser-unique-object-id
        value: ws
  - &ref_8
    id: unsubscribe
    title: Unsubscribe
    description: Remove subscriptions; same singular and batch shapes as subscribe
    type: receive
    messages:
      - &ref_19
        id: unsubscribe
        payload:
          - oneOf: &ref_2
              - type: object
                required:
                  - op
                  - subscriptions
                properties:
                  op:
                    const: unsubscribe
                    x-parser-schema-id: <anonymous-schema-19>
                  subscriptions:
                    type: array
                    minItems: 1
                    items: *ref_1
                    x-parser-schema-id: <anonymous-schema-20>
                  rid:
                    type: string
                    x-parser-schema-id: <anonymous-schema-21>
                x-parser-schema-id: <anonymous-schema-18>
              - type: object
                required:
                  - op
                  - channel
                  - filter
                properties:
                  op:
                    const: unsubscribe
                    x-parser-schema-id: <anonymous-schema-23>
                  channel:
                    type: string
                    x-parser-schema-id: <anonymous-schema-24>
                  filter:
                    type: object
                    x-parser-schema-id: <anonymous-schema-25>
                  rid:
                    type: string
                    x-parser-schema-id: <anonymous-schema-26>
                x-parser-schema-id: <anonymous-schema-22>
            x-parser-schema-id: UnsubscribeRequest
            name: unsubscribe
        headers: []
        jsonPayloadSchema:
          oneOf: *ref_2
          x-parser-schema-id: UnsubscribeRequest
        title: Unsubscribe
        example: |-
          {
            "op": "unsubscribe",
            "channel": "price.crypto",
            "filter": {
              "symbol": "btcusd"
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: unsubscribe
    bindings: []
    extensions: *ref_3
  - &ref_9
    id: auth
    title: Authenticate
    description: >-
      Unlock the gated channels on this connection with CLOB API credentials or
      a web-app ticket
    type: receive
    messages:
      - &ref_20
        id: auth
        payload:
          - name: auth
            type: object
            properties:
              - name: const
                type: string
                description: auth
                required: false
              - name: auth
                type: object
                description: >-
                  The CLOB L2 API credential trio, the same credentials used for
                  the CLOB user WebSocket.
                required: false
                properties:
                  - name: apiKey
                    type: string
                    required: true
                  - name: secret
                    type: string
                    required: true
                  - name: passphrase
                    type: string
                    required: true
              - name: rid
                type: string
                required: false
              - name: ticket
                type: string
                description: >-
                  The one-time ticket from POST /api/ws-ticket (the Polymarket
                  web app's credential), consumed by the attempt whatever the
                  outcome. Minted from a Cloudflare Turnstile solve; when the
                  widget was rendered with this connection's challenge nonce as
                  cData, the ticket verifies only on this socket, whatever IP
                  either request came from. A stale, consumed or unknown ticket,
                  or one minted for another socket, answers the non-closing
                  auth_expired: mint again.
                required: false
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - op
          properties:
            op:
              const: auth
              x-parser-schema-id: <anonymous-schema-27>
            auth:
              type: object
              required:
                - apiKey
                - secret
                - passphrase
              properties:
                apiKey:
                  type: string
                  x-parser-schema-id: <anonymous-schema-28>
                secret:
                  type: string
                  x-parser-schema-id: <anonymous-schema-29>
                passphrase:
                  type: string
                  x-parser-schema-id: <anonymous-schema-30>
              description: >-
                The CLOB L2 API credential trio, the same credentials used for
                the CLOB user WebSocket.
              x-parser-schema-id: ClobCredentials
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-31>
            ticket:
              type: string
              maxLength: 128
              description: >-
                The one-time ticket from POST /api/ws-ticket (the Polymarket web
                app's credential), consumed by the attempt whatever the outcome.
                Minted from a Cloudflare Turnstile solve; when the widget was
                rendered with this connection's challenge nonce as cData, the
                ticket verifies only on this socket, whatever IP either request
                came from. A stale, consumed or unknown ticket, or one minted
                for another socket, answers the non-closing auth_expired: mint
                again.
              x-parser-schema-id: <anonymous-schema-32>
          oneOf:
            - required:
                - auth
              x-parser-schema-id: <anonymous-schema-33>
            - required:
                - ticket
              x-parser-schema-id: <anonymous-schema-34>
          description: >-
            Exactly one credential per frame: auth (CLOB API credentials,
            third-party clients) or ticket (the web app). Zero or two answer the
            non-closing bad_op.
          x-parser-schema-id: AuthRequest
        title: Auth
        example: |-
          {
            "op": "auth",
            "rid": "a1",
            "auth": {
              "apiKey": "00000000-0000-4000-8000-000000000000",
              "secret": "<api secret>",
              "passphrase": "<api passphrase>"
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: auth
    bindings: []
    extensions: *ref_3
  - &ref_10
    id: ping
    title: Ping
    description: Optional application-level liveness probe; answered with pong
    type: receive
    messages:
      - &ref_21
        id: ping
        payload:
          - name: ping
            type: object
            properties:
              - name: const
                type: string
                description: ping
                required: false
              - name: rid
                type: string
                required: false
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - op
          properties:
            op:
              const: ping
              x-parser-schema-id: <anonymous-schema-35>
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-36>
          x-parser-schema-id: PingRequest
        title: Ping
        example: |-
          {
            "op": "ping",
            "rid": "p1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: ping
    bindings: []
    extensions: *ref_3
  - &ref_12
    id: acknowledgements
    title: Acknowledgements
    description: >-
      challenge (on request), subscribed, unsubscribed, authed, pong and error
      acks; rid is echoed when the request carried one
    type: send
    messages:
      - &ref_23
        id: challenge
        payload:
          - name: challenge
            description: >-
              This connection's challenge nonce, the answer to a challenge
              request; never pushed unasked
            type: object
            properties:
              - name: const
                type: string
                description: challenge
                required: false
              - name: nonce
                type: string
                description: >-
                  This connection's challenge nonce (a UUID). Render the
                  Turnstile widget with it as cData before minting a ticket at
                  POST /api/ws-ticket; the ticket then verifies only on this
                  socket, whatever IP either request came from. Not a secret,
                  never rotated for the life of the connection.
                required: true
              - name: rid
                type: string
                required: false
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - op
            - nonce
          properties:
            op:
              const: challenge
              x-parser-schema-id: <anonymous-schema-129>
            nonce:
              type: string
              pattern: ^[A-Za-z0-9_-]{1,255}$
              description: >-
                This connection's challenge nonce (a UUID). Render the Turnstile
                widget with it as cData before minting a ticket at POST
                /api/ws-ticket; the ticket then verifies only on this socket,
                whatever IP either request came from. Not a secret, never
                rotated for the life of the connection.
              x-parser-schema-id: <anonymous-schema-130>
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-131>
          x-parser-schema-id: ChallengeAck
        title: Challenge
        description: >-
          This connection's challenge nonce, the answer to a challenge request;
          never pushed unasked
        example: |-
          {
            "op": "challenge",
            "nonce": "3f2b6c1e-9a4d-4c2b-8e1f-0a1b2c3d4e5f",
            "rid": "c1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: challenge
      - &ref_24
        id: subscribed
        payload:
          - name: subscribed
            type: object
            properties:
              - name: op
                type: string
                enumValues:
                  - subscribed
                  - unsubscribed
                required: true
              - name: channel
                type: string
                required: true
              - name: rid
                type: string
                required: false
              - name: provider
                type: string
                description: >-
                  subscribed acks only, vendor channels only, and only while the
                  provider selector is enabled (spec 08b): the provider actually
                  SERVED - the requested pin when the catalog serves it, else
                  the lane's owner (massive is reachable on the omitted path
                  only). Also present on the idempotent duplicate-subscribe ack.
                  Absent everywhere else, including every ack while the selector
                  is disabled.
                enumValues:
                  - pyth
                  - massive
                  - chainlink
                required: false
        headers: []
        jsonPayloadSchema: &ref_4
          type: object
          required:
            - op
            - channel
          properties:
            op:
              type: string
              enum:
                - subscribed
                - unsubscribed
              x-parser-schema-id: <anonymous-schema-37>
            channel:
              type: string
              x-parser-schema-id: <anonymous-schema-38>
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-39>
            provider:
              type: string
              enum:
                - pyth
                - massive
                - chainlink
              description: >-
                subscribed acks only, vendor channels only, and only while the
                provider selector is enabled (spec 08b): the provider actually
                SERVED - the requested pin when the catalog serves it, else the
                lane's owner (massive is reachable on the omitted path only).
                Also present on the idempotent duplicate-subscribe ack. Absent
                everywhere else, including every ack while the selector is
                disabled.
              x-parser-schema-id: <anonymous-schema-40>
          x-parser-schema-id: ChannelAck
        title: Subscribed
        example: |-
          {
            "op": "subscribed",
            "channel": "price.crypto",
            "rid": "s1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: subscribed
      - &ref_25
        id: unsubscribed
        payload:
          - name: unsubscribed
            type: object
            properties:
              - name: op
                type: string
                enumValues:
                  - subscribed
                  - unsubscribed
                required: true
              - name: channel
                type: string
                required: true
              - name: rid
                type: string
                required: false
              - name: provider
                type: string
                description: >-
                  subscribed acks only, vendor channels only, and only while the
                  provider selector is enabled (spec 08b): the provider actually
                  SERVED - the requested pin when the catalog serves it, else
                  the lane's owner (massive is reachable on the omitted path
                  only). Also present on the idempotent duplicate-subscribe ack.
                  Absent everywhere else, including every ack while the selector
                  is disabled.
                enumValues:
                  - pyth
                  - massive
                  - chainlink
                required: false
        headers: []
        jsonPayloadSchema: *ref_4
        title: Unsubscribed
        example: |-
          {
            "op": "unsubscribed",
            "channel": "price.crypto"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: unsubscribed
      - &ref_26
        id: authed
        payload:
          - name: authed
            type: object
            properties:
              - name: op
                type: string
                enumValues:
                  - authed
                  - pong
                required: true
              - name: rid
                type: string
                required: false
        headers: []
        jsonPayloadSchema: &ref_5
          type: object
          required:
            - op
          properties:
            op:
              type: string
              enum:
                - authed
                - pong
              x-parser-schema-id: <anonymous-schema-41>
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-42>
          x-parser-schema-id: SimpleAck
        title: Authed
        example: |-
          {
            "op": "authed",
            "rid": "a1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: authed
      - &ref_27
        id: pong
        payload:
          - name: pong
            type: object
            properties:
              - name: op
                type: string
                enumValues:
                  - authed
                  - pong
                required: true
              - name: rid
                type: string
                required: false
        headers: []
        jsonPayloadSchema: *ref_5
        title: Pong
        example: |-
          {
            "op": "pong",
            "rid": "p1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: pong
      - &ref_28
        id: error
        payload:
          - name: error
            type: object
            properties:
              - name: const
                type: string
                description: error
                required: false
              - name: code
                type: string
                description: >-
                  bad_op malformed frame; bad_channel unknown channel;
                  bad_filter missing or invalid filter; sub_limit 65th
                  subscription (followed by close 4008); rate_limited more than
                  20 subscribe/unsubscribe frames per second (followed by close
                  4008); auth_required gated channel before auth; auth_expired
                  consumed or unknown ticket (web app lane); auth_unavailable
                  verifier unreachable, retry later; auth_invalid credentials
                  refused; auth_attempts ninth auth frame (followed by close
                  4008).
                enumValues:
                  - bad_op
                  - bad_channel
                  - bad_filter
                  - sub_limit
                  - rate_limited
                  - auth_required
                  - auth_expired
                  - auth_unavailable
                  - auth_invalid
                  - auth_attempts
                required: true
              - name: channel
                type: string
                required: false
              - name: rid
                type: string
                required: false
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - op
            - code
          properties:
            op:
              const: error
              x-parser-schema-id: <anonymous-schema-43>
            code:
              type: string
              enum:
                - bad_op
                - bad_channel
                - bad_filter
                - sub_limit
                - rate_limited
                - auth_required
                - auth_expired
                - auth_unavailable
                - auth_invalid
                - auth_attempts
              description: >-
                bad_op malformed frame; bad_channel unknown channel; bad_filter
                missing or invalid filter; sub_limit 65th subscription (followed
                by close 4008); rate_limited more than 20 subscribe/unsubscribe
                frames per second (followed by close 4008); auth_required gated
                channel before auth; auth_expired consumed or unknown ticket
                (web app lane); auth_unavailable verifier unreachable, retry
                later; auth_invalid credentials refused; auth_attempts ninth
                auth frame (followed by close 4008).
              x-parser-schema-id: <anonymous-schema-44>
            channel:
              type: string
              x-parser-schema-id: <anonymous-schema-45>
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-46>
          x-parser-schema-id: ErrorAck
        title: Error
        example: |-
          {
            "op": "error",
            "code": "auth_required",
            "channel": "price.crypto",
            "rid": "s1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: error
    bindings: []
    extensions: *ref_3
  - &ref_13
    id: pricePolymarket
    title: price.polymarket
    description: Best bid and ask per outcome token (public, no auth)
    type: send
    messages:
      - &ref_29
        id: pricePolymarket
        payload:
          - name: price.polymarket envelope
            type: object
            properties:
              - name: v
                type: integer
                description: Envelope version.
                required: true
              - name: channel
                type: string
                description: price.polymarket
                required: true
              - name: seq
                type: integer
                description: >-
                  Dense per-connection, per-channel sequence, assigned at
                  delivery. Resets on every reconnect. Snapshot frames consume a
                  seq.
                required: true
              - name: ts
                type: integer
                description: Producer event time, Unix milliseconds.
                required: true
              - name: snapshot
                type: boolean
                description: >-
                  Present and true on the one snapshot frame sent after each new
                  subscription. Absent on live frames.
                required: false
              - name: dropped
                type: integer
                description: >-
                  Number of frames dropped on this channel since the last
                  delivered frame (slow-consumer signal). Absent when zero.
                required: false
              - name: payload
                type: oneOf
                required: true
                properties:
                  - name: market
                    type: string
                    description: Condition id, 0x-prefixed.
                    required: true
                  - name: asset_id
                    type: string
                    description: Outcome token id, decimal string.
                    required: true
                  - name: best_bid
                    type: string
                    description: Decimal string price.
                    required: true
                  - name: best_ask
                    type: string
                    description: Decimal string price.
                    required: true
                  - name: hash
                    type: string
                    description: Order book hash at this change.
                    required: true
                  - name: timestamp
                    type: integer
                    description: Order book time, Unix milliseconds.
                    required: true
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - v
            - channel
            - seq
            - ts
            - payload
          properties:
            v:
              type: integer
              const: 1
              description: Envelope version.
              x-parser-schema-id: <anonymous-schema-47>
            channel:
              type: string
              const: price.polymarket
              x-parser-schema-id: <anonymous-schema-48>
            seq:
              type: integer
              description: >-
                Dense per-connection, per-channel sequence, assigned at
                delivery. Resets on every reconnect. Snapshot frames consume a
                seq.
              x-parser-schema-id: <anonymous-schema-49>
            ts:
              type: integer
              description: Producer event time, Unix milliseconds.
              x-parser-schema-id: <anonymous-schema-50>
            snapshot:
              type: boolean
              description: >-
                Present and true on the one snapshot frame sent after each new
                subscription. Absent on live frames.
              x-parser-schema-id: <anonymous-schema-51>
            dropped:
              type: integer
              description: >-
                Number of frames dropped on this channel since the last
                delivered frame (slow-consumer signal). Absent when zero.
              x-parser-schema-id: <anonymous-schema-52>
            payload:
              oneOf:
                - type: object
                  required:
                    - market
                    - asset_id
                    - best_bid
                    - best_ask
                    - hash
                    - timestamp
                  properties:
                    market:
                      type: string
                      description: Condition id, 0x-prefixed.
                      x-parser-schema-id: <anonymous-schema-53>
                    asset_id:
                      type: string
                      description: Outcome token id, decimal string.
                      x-parser-schema-id: <anonymous-schema-54>
                    best_bid:
                      type: string
                      description: Decimal string price.
                      x-parser-schema-id: <anonymous-schema-55>
                    best_ask:
                      type: string
                      description: Decimal string price.
                      x-parser-schema-id: <anonymous-schema-56>
                    hash:
                      type: string
                      description: Order book hash at this change.
                      x-parser-schema-id: <anonymous-schema-57>
                    timestamp:
                      type: integer
                      description: Order book time, Unix milliseconds.
                      x-parser-schema-id: <anonymous-schema-58>
                  x-parser-schema-id: BestBidAsk
                - type: array
                  maxItems: 0
                  description: Empty snapshot for a cold token.
                  x-parser-schema-id: <anonymous-schema-59>
              x-parser-schema-id: PricePolymarketPayload
          description: >-
            Snapshot payload is a BestBidAsk object or an empty array; live
            payload is a BestBidAsk object.
          x-parser-schema-id: PricePolymarketEnvelope
        title: price.polymarket envelope
        example: |-
          {
            "v": 1,
            "channel": "price.polymarket",
            "seq": 1,
            "ts": 1788973000123,
            "snapshot": true,
            "payload": {
              "market": "0x9deb0baac40648821f96f01339229a422e2f5c877de55dc4dbf981f95a1e709c",
              "asset_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455",
              "best_bid": "0.51",
              "best_ask": "0.53",
              "hash": "3f9c1e7a",
              "timestamp": 1788972999871
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: pricePolymarket
    bindings: []
    extensions: *ref_3
  - &ref_14
    id: priceCrypto
    title: price.crypto
    description: >-
      Pyth crypto reference prices, about one update per second per symbol
      (gated)
    type: send
    messages:
      - &ref_30
        id: priceCrypto
        payload:
          - name: price.crypto envelope
            type: object
            properties:
              - name: v
                type: integer
                description: Envelope version.
                required: true
              - name: channel
                type: string
                description: price.crypto
                required: true
              - name: seq
                type: integer
                description: >-
                  Dense per-connection, per-channel sequence, assigned at
                  delivery. Resets on every reconnect. Snapshot frames consume a
                  seq.
                required: true
              - name: ts
                type: integer
                description: Producer event time, Unix milliseconds.
                required: true
              - name: snapshot
                type: boolean
                description: >-
                  Present and true on the one snapshot frame sent after each new
                  subscription. Absent on live frames.
                required: false
              - name: dropped
                type: integer
                description: >-
                  Number of frames dropped on this channel since the last
                  delivered frame (slow-consumer signal). Absent when zero.
                required: false
              - name: payload
                type: object
                required: true
                properties:
                  - name: symbol
                    type: string
                    description: Lowercase symbol, for example btcusd.
                    required: true
                  - name: value
                    type: number
                    description: Price as a float. Prefer full_accuracy_value.
                    required: true
                  - name: full_accuracy_value
                    type: string
                    description: Exact decimal price as published by the source.
                    required: true
                  - name: timestamp
                    type: integer
                    description: Source event time, Unix milliseconds.
                    required: true
                  - name: source
                    type: string
                    description: >-
                      The producing vendor. price.crypto is a single-provider
                      lane, so the value is the topic constant.
                    enumValues:
                      - pyth
                    required: true
                  - name: symbol
                    type: string
                    required: true
                  - name: source
                    type: string
                    description: >-
                      The producing vendor, once per batch (points are
                      source-free). Topic constant on this single-provider lane.
                    enumValues:
                      - pyth
                    required: true
                  - name: data
                    type: array
                    description: >-
                      The array holds the previous two minutes of points by
                      event time and is empty for a symbol with no recent data.
                    required: true
                    properties:
                      - name: value
                        type: number
                        description: Price as a float. Prefer full_accuracy_value.
                        required: true
                      - name: full_accuracy_value
                        type: string
                        description: Exact decimal price as published by the source.
                        required: true
                      - name: timestamp
                        type: integer
                        description: Source event time, Unix milliseconds.
                        required: true
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - v
            - channel
            - seq
            - ts
            - payload
          properties:
            v:
              type: integer
              const: 1
              description: Envelope version.
              x-parser-schema-id: <anonymous-schema-60>
            channel:
              type: string
              const: price.crypto
              x-parser-schema-id: <anonymous-schema-61>
            seq:
              type: integer
              description: >-
                Dense per-connection, per-channel sequence, assigned at
                delivery. Resets on every reconnect. Snapshot frames consume a
                seq.
              x-parser-schema-id: <anonymous-schema-62>
            ts:
              type: integer
              description: Producer event time, Unix milliseconds.
              x-parser-schema-id: <anonymous-schema-63>
            snapshot:
              type: boolean
              description: >-
                Present and true on the one snapshot frame sent after each new
                subscription. Absent on live frames.
              x-parser-schema-id: <anonymous-schema-64>
            dropped:
              type: integer
              description: >-
                Number of frames dropped on this channel since the last
                delivered frame (slow-consumer signal). Absent when zero.
              x-parser-schema-id: <anonymous-schema-65>
            payload:
              oneOf:
                - type: object
                  required:
                    - symbol
                    - value
                    - full_accuracy_value
                    - timestamp
                    - source
                  properties:
                    symbol:
                      type: string
                      description: Lowercase symbol, for example btcusd.
                      x-parser-schema-id: <anonymous-schema-66>
                    value:
                      type: number
                      description: Price as a float. Prefer full_accuracy_value.
                      x-parser-schema-id: <anonymous-schema-67>
                    full_accuracy_value:
                      type: string
                      description: Exact decimal price as published by the source.
                      x-parser-schema-id: <anonymous-schema-68>
                    timestamp:
                      type: integer
                      description: Source event time, Unix milliseconds.
                      x-parser-schema-id: <anonymous-schema-69>
                    source:
                      type: string
                      enum:
                        - pyth
                      description: >-
                        The producing vendor. price.crypto is a single-provider
                        lane, so the value is the topic constant.
                      x-parser-schema-id: <anonymous-schema-70>
                  x-parser-schema-id: CryptoPoint
                - type: object
                  required:
                    - symbol
                    - source
                    - data
                  properties:
                    symbol:
                      type: string
                      x-parser-schema-id: <anonymous-schema-72>
                    source:
                      type: string
                      enum:
                        - pyth
                      description: >-
                        The producing vendor, once per batch (points are
                        source-free). Topic constant on this single-provider
                        lane.
                      x-parser-schema-id: <anonymous-schema-73>
                    data:
                      type: array
                      items: &ref_6
                        type: object
                        required:
                          - value
                          - full_accuracy_value
                          - timestamp
                        properties:
                          value:
                            type: number
                            description: Price as a float. Prefer full_accuracy_value.
                            x-parser-schema-id: <anonymous-schema-75>
                          full_accuracy_value:
                            type: string
                            description: Exact decimal price as published by the source.
                            x-parser-schema-id: <anonymous-schema-76>
                          timestamp:
                            type: integer
                            description: Source event time, Unix milliseconds.
                            x-parser-schema-id: <anonymous-schema-77>
                        description: >-
                          One historical point of a snapshot batch. Batch points
                          carry NO symbol and NO source: both live once at batch
                          level (the batch object's own fields), matching what
                          the server actually emits - the retired shared point
                          schema wrongly REQUIRED a symbol on every point.
                        x-parser-schema-id: BatchPoint
                      description: >-
                        The array holds the previous two minutes of points by
                        event time and is empty for a symbol with no recent
                        data.
                      x-parser-schema-id: <anonymous-schema-74>
                  x-parser-schema-id: <anonymous-schema-71>
              x-parser-schema-id: PriceCryptoPayload
          description: Snapshot payload is a symbol batch; live payload is a single point.
          x-parser-schema-id: PriceCryptoEnvelope
        title: price.crypto envelope
        example: |-
          {
            "v": 1,
            "channel": "price.crypto",
            "seq": 1,
            "ts": 1788973000000,
            "snapshot": true,
            "payload": {
              "symbol": "btcusd",
              "source": "pyth",
              "data": [
                {
                  "timestamp": 1788972880000,
                  "value": 64123.5,
                  "full_accuracy_value": "64123.50000000"
                },
                {
                  "timestamp": 1788972881000,
                  "value": 64125.1,
                  "full_accuracy_value": "64125.10000000"
                }
              ]
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: priceCrypto
    bindings: []
    extensions: *ref_3
  - &ref_15
    id: priceEquity
    title: price.equity
    description: Pyth equity, ETF, forex, metals and commodity reference prices (gated)
    type: send
    messages:
      - &ref_31
        id: priceEquity
        payload:
          - name: price.equity envelope
            type: object
            properties:
              - name: v
                type: integer
                description: Envelope version.
                required: true
              - name: channel
                type: string
                description: price.equity
                required: true
              - name: seq
                type: integer
                description: >-
                  Dense per-connection, per-channel sequence, assigned at
                  delivery. Resets on every reconnect. Snapshot frames consume a
                  seq.
                required: true
              - name: ts
                type: integer
                description: Producer event time, Unix milliseconds.
                required: true
              - name: snapshot
                type: boolean
                description: >-
                  Present and true on the one snapshot frame sent after each new
                  subscription. Absent on live frames.
                required: false
              - name: dropped
                type: integer
                description: >-
                  Number of frames dropped on this channel since the last
                  delivered frame (slow-consumer signal). Absent when zero.
                required: false
              - name: payload
                type: object
                required: true
                properties:
                  - name: symbol
                    type: string
                    description: Lowercase symbol, for example btcusd or aapl.
                    required: true
                  - name: value
                    type: number
                    description: Price as a float. Prefer full_accuracy_value.
                    required: true
                  - name: full_accuracy_value
                    type: string
                    description: Exact decimal price as published by the source.
                    required: true
                  - name: timestamp
                    type: integer
                    description: Source event time, Unix milliseconds.
                    required: true
                  - name: source
                    type: string
                    description: >-
                      Vendor that produced this price, on every point this
                      server emits. A per-symbol ownership flip rolls the edge
                      fleet, so mid-roll a symbol's consecutive frames can
                      legitimately carry different sources until the roll
                      completes.
                    enumValues:
                      - pyth
                      - massive
                      - chainlink
                    required: true
                  - name: received_at
                    type: integer
                    description: Ingest receive time, Unix milliseconds. Live frames only.
                    required: false
                  - name: is_carried_forward
                    type: boolean
                    description: >-
                      True when the value is carried forward from the last
                      session (market closed). Live frames only.
                    required: false
                  - name: symbol
                    type: string
                    required: true
                  - name: source
                    type: string
                    description: >-
                      Vendor that produced every point in this batch, once at
                      batch level: symbol ownership is fixed for the life of a
                      pod, so one series never mixes vendors. After a reconnect
                      during an ownership flip, the batch can carry a different
                      source than the previous connection's frames.
                    enumValues:
                      - pyth
                      - massive
                      - chainlink
                    required: true
                  - name: data
                    type: array
                    description: >-
                      The array holds the previous two minutes of points by
                      event time and is empty for a symbol with no recent data.
                    required: true
                    properties:
                      - name: value
                        type: number
                        description: Price as a float. Prefer full_accuracy_value.
                        required: true
                      - name: full_accuracy_value
                        type: string
                        description: Exact decimal price as published by the source.
                        required: true
                      - name: timestamp
                        type: integer
                        description: Source event time, Unix milliseconds.
                        required: true
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - v
            - channel
            - seq
            - ts
            - payload
          properties:
            v:
              type: integer
              const: 1
              description: Envelope version.
              x-parser-schema-id: <anonymous-schema-78>
            channel:
              type: string
              const: price.equity
              x-parser-schema-id: <anonymous-schema-79>
            seq:
              type: integer
              description: >-
                Dense per-connection, per-channel sequence, assigned at
                delivery. Resets on every reconnect. Snapshot frames consume a
                seq.
              x-parser-schema-id: <anonymous-schema-80>
            ts:
              type: integer
              description: Producer event time, Unix milliseconds.
              x-parser-schema-id: <anonymous-schema-81>
            snapshot:
              type: boolean
              description: >-
                Present and true on the one snapshot frame sent after each new
                subscription. Absent on live frames.
              x-parser-schema-id: <anonymous-schema-82>
            dropped:
              type: integer
              description: >-
                Number of frames dropped on this channel since the last
                delivered frame (slow-consumer signal). Absent when zero.
              x-parser-schema-id: <anonymous-schema-83>
            payload:
              oneOf:
                - type: object
                  required:
                    - symbol
                    - value
                    - full_accuracy_value
                    - timestamp
                    - source
                  properties:
                    symbol:
                      type: string
                      description: Lowercase symbol, for example btcusd or aapl.
                      x-parser-schema-id: <anonymous-schema-84>
                    value:
                      type: number
                      description: Price as a float. Prefer full_accuracy_value.
                      x-parser-schema-id: <anonymous-schema-85>
                    full_accuracy_value:
                      type: string
                      description: Exact decimal price as published by the source.
                      x-parser-schema-id: <anonymous-schema-86>
                    timestamp:
                      type: integer
                      description: Source event time, Unix milliseconds.
                      x-parser-schema-id: <anonymous-schema-87>
                    source:
                      type: string
                      enum:
                        - pyth
                        - massive
                        - chainlink
                      description: >-
                        Vendor that produced this price, on every point this
                        server emits. A per-symbol ownership flip rolls the edge
                        fleet, so mid-roll a symbol's consecutive frames can
                        legitimately carry different sources until the roll
                        completes.
                      x-parser-schema-id: <anonymous-schema-88>
                    received_at:
                      type: integer
                      description: >-
                        Ingest receive time, Unix milliseconds. Live frames
                        only.
                      x-parser-schema-id: <anonymous-schema-89>
                    is_carried_forward:
                      type: boolean
                      description: >-
                        True when the value is carried forward from the last
                        session (market closed). Live frames only.
                      x-parser-schema-id: <anonymous-schema-90>
                  x-parser-schema-id: EquityPoint
                - type: object
                  required:
                    - symbol
                    - source
                    - data
                  properties:
                    symbol:
                      type: string
                      x-parser-schema-id: <anonymous-schema-92>
                    source:
                      type: string
                      enum:
                        - pyth
                        - massive
                        - chainlink
                      description: >-
                        Vendor that produced every point in this batch, once at
                        batch level: symbol ownership is fixed for the life of a
                        pod, so one series never mixes vendors. After a
                        reconnect during an ownership flip, the batch can carry
                        a different source than the previous connection's
                        frames.
                      x-parser-schema-id: <anonymous-schema-93>
                    data:
                      type: array
                      items: *ref_6
                      description: >-
                        The array holds the previous two minutes of points by
                        event time and is empty for a symbol with no recent
                        data.
                      x-parser-schema-id: <anonymous-schema-94>
                  x-parser-schema-id: <anonymous-schema-91>
              x-parser-schema-id: PriceEquityPayload
          description: Snapshot payload is a symbol batch; live payload is a single point.
          x-parser-schema-id: PriceEquityEnvelope
        title: price.equity envelope
        example: |-
          {
            "v": 1,
            "channel": "price.equity",
            "seq": 5,
            "ts": 1788973001200,
            "payload": {
              "symbol": "aapl",
              "value": 189.42,
              "full_accuracy_value": "189.4217",
              "timestamp": 1788973001200,
              "received_at": 1788973001207,
              "is_carried_forward": false,
              "source": "pyth"
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: priceEquity
    bindings: []
    extensions: *ref_3
  - &ref_16
    id: priceCryptoTwap
    title: price.crypto.twap
    description: >-
      Chainlink time-weighted average prices used by crypto up/down market
      resolution (gated)
    type: send
    messages:
      - &ref_32
        id: priceCryptoTwap
        payload:
          - name: price.crypto.twap envelope
            type: object
            properties:
              - name: v
                type: integer
                description: Envelope version.
                required: true
              - name: channel
                type: string
                description: price.crypto.twap
                required: true
              - name: seq
                type: integer
                description: >-
                  Dense per-connection, per-channel sequence, assigned at
                  delivery. Resets on every reconnect. Snapshot frames consume a
                  seq.
                required: true
              - name: ts
                type: integer
                description: Producer event time, Unix milliseconds.
                required: true
              - name: snapshot
                type: boolean
                description: >-
                  Present and true on the one snapshot frame sent after each new
                  subscription. Absent on live frames.
                required: false
              - name: dropped
                type: integer
                description: >-
                  Number of frames dropped on this channel since the last
                  delivered frame (slow-consumer signal). Absent when zero.
                required: false
              - name: payload
                type: object
                required: true
                properties:
                  - name: symbol
                    type: string
                    description: Lowercase symbol, for example btcusd.
                    required: true
                  - name: value
                    type: number
                    description: Price as a float. Prefer full_accuracy_value.
                    required: true
                  - name: full_accuracy_value
                    type: string
                    description: Exact decimal price as published by the source.
                    required: true
                  - name: timestamp
                    type: integer
                    description: Source event time, Unix milliseconds.
                    required: true
                  - name: window_seconds
                    type: integer
                    description: Averaging window of the TWAP series. Only 60 exists today.
                    required: true
                  - name: source
                    type: string
                    description: >-
                      The producing vendor. price.crypto.twap is a
                      single-provider lane, so the value is the topic constant.
                    enumValues:
                      - chainlink
                    required: true
                  - name: symbol
                    type: string
                    required: true
                  - name: data
                    type: array
                    description: >-
                      The array holds the previous two minutes of points by
                      event time and is empty for a symbol with no recent data.
                    required: true
                    properties:
                      - name: value
                        type: number
                        description: Price as a float. Prefer full_accuracy_value.
                        required: true
                      - name: full_accuracy_value
                        type: string
                        description: Exact decimal price as published by the source.
                        required: true
                      - name: timestamp
                        type: integer
                        description: Source event time, Unix milliseconds.
                        required: true
                  - name: window_seconds
                    type: integer
                    description: 60
                    required: true
                  - name: source
                    type: string
                    description: >-
                      The producing vendor, once per batch (points are
                      source-free). Topic constant on this single-provider lane.
                    enumValues:
                      - chainlink
                    required: true
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - v
            - channel
            - seq
            - ts
            - payload
          properties:
            v:
              type: integer
              const: 1
              description: Envelope version.
              x-parser-schema-id: <anonymous-schema-95>
            channel:
              type: string
              const: price.crypto.twap
              x-parser-schema-id: <anonymous-schema-96>
            seq:
              type: integer
              description: >-
                Dense per-connection, per-channel sequence, assigned at
                delivery. Resets on every reconnect. Snapshot frames consume a
                seq.
              x-parser-schema-id: <anonymous-schema-97>
            ts:
              type: integer
              description: Producer event time, Unix milliseconds.
              x-parser-schema-id: <anonymous-schema-98>
            snapshot:
              type: boolean
              description: >-
                Present and true on the one snapshot frame sent after each new
                subscription. Absent on live frames.
              x-parser-schema-id: <anonymous-schema-99>
            dropped:
              type: integer
              description: >-
                Number of frames dropped on this channel since the last
                delivered frame (slow-consumer signal). Absent when zero.
              x-parser-schema-id: <anonymous-schema-100>
            payload:
              oneOf:
                - type: object
                  required:
                    - symbol
                    - value
                    - full_accuracy_value
                    - timestamp
                    - window_seconds
                    - source
                  properties:
                    symbol:
                      type: string
                      description: Lowercase symbol, for example btcusd.
                      x-parser-schema-id: <anonymous-schema-101>
                    value:
                      type: number
                      description: Price as a float. Prefer full_accuracy_value.
                      x-parser-schema-id: <anonymous-schema-102>
                    full_accuracy_value:
                      type: string
                      description: Exact decimal price as published by the source.
                      x-parser-schema-id: <anonymous-schema-103>
                    timestamp:
                      type: integer
                      description: Source event time, Unix milliseconds.
                      x-parser-schema-id: <anonymous-schema-104>
                    window_seconds:
                      type: integer
                      const: 60
                      description: >-
                        Averaging window of the TWAP series. Only 60 exists
                        today.
                      x-parser-schema-id: <anonymous-schema-105>
                    source:
                      type: string
                      enum:
                        - chainlink
                      description: >-
                        The producing vendor. price.crypto.twap is a
                        single-provider lane, so the value is the topic
                        constant.
                      x-parser-schema-id: <anonymous-schema-106>
                  x-parser-schema-id: TwapPoint
                - type: object
                  required:
                    - symbol
                    - data
                    - window_seconds
                    - source
                  properties:
                    symbol:
                      type: string
                      x-parser-schema-id: <anonymous-schema-108>
                    data:
                      type: array
                      items: *ref_6
                      description: >-
                        The array holds the previous two minutes of points by
                        event time and is empty for a symbol with no recent
                        data.
                      x-parser-schema-id: <anonymous-schema-109>
                    window_seconds:
                      type: integer
                      const: 60
                      x-parser-schema-id: <anonymous-schema-110>
                    source:
                      type: string
                      enum:
                        - chainlink
                      description: >-
                        The producing vendor, once per batch (points are
                        source-free). Topic constant on this single-provider
                        lane.
                      x-parser-schema-id: <anonymous-schema-111>
                  x-parser-schema-id: <anonymous-schema-107>
              x-parser-schema-id: PriceCryptoTwapPayload
          description: >-
            Snapshot payload is a symbol batch carrying window_seconds; live
            payload is a single point carrying window_seconds.
          x-parser-schema-id: PriceCryptoTwapEnvelope
        title: price.crypto.twap envelope
        example: |-
          {
            "v": 1,
            "channel": "price.crypto.twap",
            "seq": 3,
            "ts": 1788973002000,
            "payload": {
              "symbol": "btcusd",
              "value": 64120.7,
              "full_accuracy_value": "64120.70000000",
              "timestamp": 1788973002000,
              "window_seconds": 60,
              "source": "chainlink"
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: priceCryptoTwap
    bindings: []
    extensions: *ref_3
  - &ref_17
    id: priceEquityTwap
    title: price.equity.twap
    description: Chainlink Data Streams FX time-weighted average prices (gated)
    type: send
    messages:
      - &ref_33
        id: priceEquityTwap
        payload:
          - name: price.equity.twap envelope
            type: object
            properties:
              - name: v
                type: integer
                description: Envelope version.
                required: true
              - name: channel
                type: string
                description: price.equity.twap
                required: true
              - name: seq
                type: integer
                description: >-
                  Dense per-connection, per-channel sequence, assigned at
                  delivery. Resets on every reconnect. Snapshot frames consume a
                  seq.
                required: true
              - name: ts
                type: integer
                description: Producer event time, Unix milliseconds.
                required: true
              - name: snapshot
                type: boolean
                description: >-
                  Present and true on the one snapshot frame sent after each new
                  subscription. Absent on live frames.
                required: false
              - name: dropped
                type: integer
                description: >-
                  Number of frames dropped on this channel since the last
                  delivered frame (slow-consumer signal). Absent when zero.
                required: false
              - name: payload
                type: object
                required: true
                properties:
                  - name: symbol
                    type: string
                    description: Lowercase symbol, for example eurusd.
                    required: true
                  - name: value
                    type: number
                    description: Price as a float. Prefer full_accuracy_value.
                    required: true
                  - name: full_accuracy_value
                    type: string
                    description: Exact decimal price as published by the source.
                    required: true
                  - name: timestamp
                    type: integer
                    description: Source event time, Unix milliseconds.
                    required: true
                  - name: window_seconds
                    type: integer
                    description: Averaging window of the TWAP series. Only 60 exists today.
                    required: true
                  - name: source
                    type: string
                    description: >-
                      Vendor that produced this price, on every point this
                      server emits. The channel is Chainlink-only today, so the
                      emitted value is chainlink. Kept as its own schema because
                      this enum is the full vendor set while TwapPoint pins its
                      topic constant.
                    enumValues:
                      - pyth
                      - massive
                      - chainlink
                    required: true
                  - name: symbol
                    type: string
                    required: true
                  - name: source
                    type: string
                    description: >-
                      Vendor that produced every point in this batch, once at
                      batch level. The channel is Chainlink-only today, so the
                      value is chainlink.
                    enumValues:
                      - pyth
                      - massive
                      - chainlink
                    required: true
                  - name: data
                    type: array
                    description: >-
                      The array holds the previous two minutes of points by
                      event time and is empty for a symbol with no recent data.
                    required: true
                    properties:
                      - name: value
                        type: number
                        description: Price as a float. Prefer full_accuracy_value.
                        required: true
                      - name: full_accuracy_value
                        type: string
                        description: Exact decimal price as published by the source.
                        required: true
                      - name: timestamp
                        type: integer
                        description: Source event time, Unix milliseconds.
                        required: true
                  - name: window_seconds
                    type: integer
                    description: 60
                    required: true
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - v
            - channel
            - seq
            - ts
            - payload
          properties:
            v:
              type: integer
              const: 1
              description: Envelope version.
              x-parser-schema-id: <anonymous-schema-112>
            channel:
              type: string
              const: price.equity.twap
              x-parser-schema-id: <anonymous-schema-113>
            seq:
              type: integer
              description: >-
                Dense per-connection, per-channel sequence, assigned at
                delivery. Resets on every reconnect. Snapshot frames consume a
                seq.
              x-parser-schema-id: <anonymous-schema-114>
            ts:
              type: integer
              description: Producer event time, Unix milliseconds.
              x-parser-schema-id: <anonymous-schema-115>
            snapshot:
              type: boolean
              description: >-
                Present and true on the one snapshot frame sent after each new
                subscription. Absent on live frames.
              x-parser-schema-id: <anonymous-schema-116>
            dropped:
              type: integer
              description: >-
                Number of frames dropped on this channel since the last
                delivered frame (slow-consumer signal). Absent when zero.
              x-parser-schema-id: <anonymous-schema-117>
            payload:
              oneOf:
                - type: object
                  required:
                    - symbol
                    - value
                    - full_accuracy_value
                    - timestamp
                    - window_seconds
                    - source
                  properties:
                    symbol:
                      type: string
                      description: Lowercase symbol, for example eurusd.
                      x-parser-schema-id: <anonymous-schema-118>
                    value:
                      type: number
                      description: Price as a float. Prefer full_accuracy_value.
                      x-parser-schema-id: <anonymous-schema-119>
                    full_accuracy_value:
                      type: string
                      description: Exact decimal price as published by the source.
                      x-parser-schema-id: <anonymous-schema-120>
                    timestamp:
                      type: integer
                      description: Source event time, Unix milliseconds.
                      x-parser-schema-id: <anonymous-schema-121>
                    window_seconds:
                      type: integer
                      const: 60
                      description: >-
                        Averaging window of the TWAP series. Only 60 exists
                        today.
                      x-parser-schema-id: <anonymous-schema-122>
                    source:
                      type: string
                      enum:
                        - pyth
                        - massive
                        - chainlink
                      description: >-
                        Vendor that produced this price, on every point this
                        server emits. The channel is Chainlink-only today, so
                        the emitted value is chainlink. Kept as its own schema
                        because this enum is the full vendor set while TwapPoint
                        pins its topic constant.
                      x-parser-schema-id: <anonymous-schema-123>
                  x-parser-schema-id: EquityTwapPoint
                - type: object
                  required:
                    - symbol
                    - source
                    - data
                    - window_seconds
                  properties:
                    symbol:
                      type: string
                      x-parser-schema-id: <anonymous-schema-125>
                    source:
                      type: string
                      enum:
                        - pyth
                        - massive
                        - chainlink
                      description: >-
                        Vendor that produced every point in this batch, once at
                        batch level. The channel is Chainlink-only today, so the
                        value is chainlink.
                      x-parser-schema-id: <anonymous-schema-126>
                    data:
                      type: array
                      items: *ref_6
                      description: >-
                        The array holds the previous two minutes of points by
                        event time and is empty for a symbol with no recent
                        data.
                      x-parser-schema-id: <anonymous-schema-127>
                    window_seconds:
                      type: integer
                      const: 60
                      x-parser-schema-id: <anonymous-schema-128>
                  x-parser-schema-id: <anonymous-schema-124>
              x-parser-schema-id: PriceEquityTwapPayload
          description: >-
            Snapshot payload is a symbol batch carrying window_seconds; live
            payload is a single point carrying window_seconds.
          x-parser-schema-id: PriceEquityTwapEnvelope
        title: price.equity.twap envelope
        example: |-
          {
            "v": 1,
            "channel": "price.equity.twap",
            "seq": 3,
            "ts": 1788973002000,
            "payload": {
              "symbol": "eurusd",
              "value": 1.0842,
              "full_accuracy_value": "1.08420000",
              "timestamp": 1788973002000,
              "window_seconds": 60,
              "source": "chainlink"
            }
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: priceEquityTwap
    bindings: []
    extensions: *ref_3
  - &ref_11
    id: challenge
    title: Request the challenge
    description: Ask for this connection's challenge nonce
    type: receive
    messages:
      - &ref_22
        id: challengeRequest
        payload:
          - name: challenge (request)
            description: Ask for this connection's challenge nonce
            type: object
            properties:
              - name: const
                type: string
                description: challenge
                required: false
              - name: rid
                type: string
                required: false
        headers: []
        jsonPayloadSchema:
          type: object
          required:
            - op
          properties:
            op:
              const: challenge
              x-parser-schema-id: <anonymous-schema-132>
            rid:
              type: string
              x-parser-schema-id: <anonymous-schema-133>
          x-parser-schema-id: ChallengeRequest
        title: challenge (request)
        description: Ask for this connection's challenge nonce
        example: |-
          {
            "op": "challenge",
            "rid": "c1"
          }
        bindings: []
        extensions:
          - id: x-parser-unique-object-id
            value: challengeRequest
    bindings: []
    extensions: *ref_3
sendOperations:
  - *ref_7
  - *ref_8
  - *ref_9
  - *ref_10
  - *ref_11
receiveOperations:
  - *ref_12
  - *ref_13
  - *ref_14
  - *ref_15
  - *ref_16
  - *ref_17
sendMessages:
  - *ref_18
  - *ref_19
  - *ref_20
  - *ref_21
  - *ref_22
receiveMessages:
  - *ref_23
  - *ref_24
  - *ref_25
  - *ref_26
  - *ref_27
  - *ref_28
  - *ref_29
  - *ref_30
  - *ref_31
  - *ref_32
  - *ref_33
extensions:
  - id: x-parser-unique-object-id
    value: ws
securitySchemes: []

````