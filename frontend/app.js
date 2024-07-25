document.getElementById('create-identity-form').addEventListener('submit', async function(event) {
    event.preventDefault();
    const name = document.getElementById('name').value;
    const email = document.getElementById('email').value;
    const response = await fetch('http://localhost:3030/identity', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ name, email })
    });
    const result = await response.json();
    alert(`Identity Created: ${JSON.stringify(result)}`);
});

document.getElementById('submit-transaction-form').addEventListener('submit', async function(event) {
    event.preventDefault();
    const from = document.getElementById('from').value;
    const to = document.getElementById('to').value;
    const amount = parseFloat(document.getElementById('amount').value);
    const currency = document.getElementById('currency').value;
    const response = await fetch('http://localhost:3030/transaction', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ from, to, amount, currency_type: currency, timestamp: Date.now(), signature: null })
    });
    if (response.ok) {
        alert('Transaction Submitted');
    } else {
        alert('Error Submitting Transaction');
    }
});

document.getElementById('create-proposal-form').addEventListener('submit', async function(event) {
    event.preventDefault();
    const title = document.getElementById('title').value;
    const description = document.getElementById('description').value;
    const proposer = document.getElementById('proposer').value;
    const response = await fetch('http://localhost:3030/proposal', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            id: Date.now().toString(),
            title,
            description,
            proposer,
            created_at: new Date().toISOString(),
            voting_ends_at: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(), // 1 week from now
            status: 'Active',
            proposal_type: 'Constitutional',
            category: 'Economic',
            required_quorum: 0.66,
            execution_timestamp: null
        })
    });
    if (response.ok) {
        alert('Proposal Created');
    } else {
        alert('Error Creating Proposal');
    }
});
