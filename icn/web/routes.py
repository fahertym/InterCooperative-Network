from flask import render_template, request, redirect, url_for, flash
from . import app
from ..blockchain import chain  # Changed this import
from ..dao.governance import VotingStrategy, ProposalType

blockchain = chain.Blockchain()  # Changed this line

@app.route('/')
def index():
    cooperatives = blockchain.cooperative_manager.list_cooperatives()
    return render_template('index.html', cooperatives=cooperatives)

# ... (rest of the routes remain the same)
@app.route('/create_cooperative', methods=['GET', 'POST'])
def create_cooperative():
    if request.method == 'POST':
        name = request.form['name']
        coop = blockchain.create_cooperative(name)
        if coop:
            flash(f"Cooperative '{name}' created successfully.", 'success')
            return redirect(url_for('index'))
        else:
            flash(f"Failed to create cooperative '{name}'.", 'error')
    return render_template('create_cooperative.html')

@app.route('/cooperative/<name>')
def cooperative_details(name):
    coop = blockchain.get_cooperative(name)
    if coop:
        return render_template('cooperative_details.html', cooperative=coop)
    flash(f"Cooperative '{name}' not found.", 'error')
    return redirect(url_for('index'))

@app.route('/create_proposal/<coop_name>', methods=['GET', 'POST'])
def create_proposal(coop_name):
    coop = blockchain.get_cooperative(coop_name)
    if not coop:
        flash(f"Cooperative '{coop_name}' not found.", 'error')
        return redirect(url_for('index'))

    if request.method == 'POST':
        creator = request.form['creator']
        description = request.form['description']
        proposal_type = ProposalType[request.form['proposal_type']]
        voting_period = int(request.form['voting_period'])
        voting_strategy = VotingStrategy[request.form['voting_strategy']]
        required_majority = float(request.form['required_majority'])

        proposal_id = coop.create_proposal(creator, description, proposal_type, voting_period, voting_strategy, required_majority)
        if proposal_id is not None:
            flash(f"Proposal created with ID: {proposal_id}", 'success')
            return redirect(url_for('cooperative_details', name=coop_name))
        else:
            flash("Failed to create proposal.", 'error')

    return render_template('create_proposal.html', cooperative=coop)