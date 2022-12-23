# BACKUP

Section 2: Add a description field
----------------------------------

- Create a new branch
- Update app.yml to add new field in TodoItem model
- Update app to point to the new branch
- Sync
- Access the API and check the description
- Update CLI to accept and show description

Section 3: Authorization

- Create a new branch
- Toggle default access in app.yml
- Update app to point to the new branch
- Sync
- Try todo-cli with the new API
- Testing access the API with admin token using curl
- Update todo-cli to pass token
- Refactor using argparse
- Add check_response
- Add --token

Section 4: Multi-tenant

- Creating a user in the app database using curl command
- Try accessing API as a normal user
- Permission denied
- Create a new branch in the app repo
- Update model visibility_scope and API access
- Update app to point to the new branch
- Sync
- Using the user’s token to access API

Section 5: 


Topics:

- Multi list
- Reference, foreign key
- Shared lists
- Attachments
- Different field types of items
- Datetime
- User registration
- Third party auth (Google, GitHub)
