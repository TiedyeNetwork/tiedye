const express = require('express');

const app = express();
const port = 7666;

app.get('/mock', (_, res) => {
  console.log('was hit!');

  const data = Buffer.from('2a', 'hex');
  // console.log(data.toString('hex'));
  res.status(200).send(data);
});

app.listen(port, () => console.log(`Started the oracle service on port ${port}.`));
