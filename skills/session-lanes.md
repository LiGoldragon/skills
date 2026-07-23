# Skill — session lanes

- Give each editing worker an assigned session, lane, and mode.
- Register before writes and claim exact paths.
- Use Recovery only for the matching active lane.
- Release owned claims and unregister at closeout.
