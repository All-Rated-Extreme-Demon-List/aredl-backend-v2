**Bounty Board API Solution**

To implement the Bounty Board API, we will create a RESTful API with the following endpoints:

### Endpoints

1. **GET /bounty-board**: Returns the list of levels on the bounty board, divided by difficulty (easy, medium, hard)
2. **POST /bounty-board**: Adds a new level to the bounty board
3. **DELETE /bounty-board/:levelId**: Removes a level from the bounty board
4. **GET /bounty-board/:levelId**: Returns the details of a specific level on the bounty board
5. **POST /bounty-board/:levelId/submissions**: Submits a new record for a level on the bounty board
6. **GET /players/:playerId/bounty-board**: Returns the list of bounty board levels completed by a player
7. **GET /players/:playerId/badges**: Returns the list of badges earned by a player

### Database Schema

We will use a MongoDB database to store the bounty board data. The schema will consist of the following collections:

1. **levels**: stores information about each level on the bounty board
	* _id (ObjectId)
	* name (String)
	* difficulty (String: easy, medium, hard)
	* targetCount (Number)
	* submissions (Array of submission objects)
2. **submissions**: stores information about each submission for a level
	* _id (ObjectId)
	* levelId (ObjectId)
	* playerId (ObjectId)
	* timestamp (Date)
3. **players**: stores information about each player
	* _id (ObjectId)
	* name (String)
	* bountyBoardCompletions (Number)
	* badges (Array of badge objects)
4. **badges**: stores information about each badge
	* _id (ObjectId)
	* name (String)
	* description (String)

### API Implementation

We will use Node.js and Express.js to implement the API. Here is an example of how the API endpoints can be implemented:
```javascript
const express = require('express');
const app = express();
const mongoose = require('mongoose');

// Connect to MongoDB
mongoose.connect('mongodb://localhost/aredl-backend-v2', { useNewUrlParser: true, useUnifiedTopology: true });

// Define the level model
const levelSchema = new mongoose.Schema({
  name: String,
  difficulty: String,
  targetCount: Number,
  submissions: [{ type: mongoose.Schema.Types.ObjectId, ref: 'Submission' }]
});
const Level = mongoose.model('Level', levelSchema);

// Define the submission model
const submissionSchema = new mongoose.Schema({
  levelId: { type: mongoose.Schema.Types.ObjectId, ref: 'Level' },
  playerId: { type: mongoose.Schema.Types.ObjectId, ref: 'Player' },
  timestamp: Date
});
const Submission = mongoose.model('Submission', submissionSchema);

// Define the player model
const playerSchema = new mongoose.Schema({
  name: String,
  bountyBoardCompletions: Number,
  badges: [{ type: mongoose.Schema.Types.ObjectId, ref: 'Badge' }]
});
const Player = mongoose.model('Player', playerSchema);

// Define the badge model
const badgeSchema = new mongoose.Schema({
  name: String,
  description: String
});
const Badge = mongoose.model('Badge', badgeSchema);

// Implement API endpoints
app.get('/bounty-board', async (req, res) => {
  const levels = await Level.find().populate('submissions');
  res.json(levels);
});

app.post('/bounty-board', async (req, res) => {
  const level = new Level(req.body);
  await level.save();
  res.json(level);
});

app.delete('/bounty-board/:levelId', async (req, res) => {
  await Level.findByIdAndRemove(req.params.levelId);
  res.json({ message: 'Level removed from bounty board' });
});

app.get('/bounty-board/:levelId', async (req, res) => {
  const level = await Level.findById(req.params.levelId).populate('submissions');
  res.json(level);
});

app.post('/bounty-board/:levelId/submissions', async (req, res) => {
  const submission = new Submission(req.body);
  await submission.save();
  res.json(submission);
});

app.get('/players/:playerId/bounty-board', async (req, res) => {
  const player = await Player.findById(req.params.playerId).populate('bountyBoardCompletions');
  res.json(player.bountyBoardCompletions);
});

app.get('/players/:playerId/badges', async (req, res) => {
  const player = await Player.findById(req.params.playerId).populate('badges');
  res.json(player.badges);
});
```
This implementation provides a basic structure for the Bounty Board API. However, it is just a starting point, and you will need to add additional functionality, error handling, and security measures to make it production-ready.

**Example Use Cases**

1. Adding a new level to the bounty board:
```bash
curl -X POST \
  http://localhost:3000/bounty-board \
  -H 'Content-Type: application/json' \
  -d '{"name": "New Level", "difficulty": "easy", "targetCount": 10}'
```
2. Submitting a new record for a level on the bounty board:
```bash
curl -X POST \
  http://localhost:3000/bounty-board/12345/submissions \
  -H 'Content-Type: application/json' \
  -d '{"playerId": "67890", "timestamp": "2023-03-01T12:00:00.000Z"}'
```
3. Retrieving the list of bounty board levels completed by a player:
```bash
curl -X GET \
  http://localhost:3000/players/67890/bounty-board
```
Note: This is just a basic example, and you will need to modify it to fit your specific use case. Additionally, you will need to implement authentication and authorization to secure the API.