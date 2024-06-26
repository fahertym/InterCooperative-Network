from flask import render_template, request, redirect, url_for, flash, jsonify, session
from . import app
from ..blockchain import chain
from ..dao.governance import VotingStrategy, ProposalType

blockchain = chain.Blockchain()

@app.route('/')
def index():
    cooperatives = blockchain.cooperative_manager.list_cooperatives()
    return render_template('index.html', cooperatives=cooperatives)

@app.route('/create_cooperative', methods=['GET', 'POST'])
def create_cooperative():
    if request.method == 'POST':
        name = request.form['name']
        try:
            coop = blockchain.create_cooperative(name)
            if coop:
                flash(f"Cooperative '{name}' created successfully.", 'success')
                return redirect(url_for('index'))
            else:
                flash(f"Failed to create cooperative '{name}'.", 'error')
        except Exception as e:
            flash(f"Error creating cooperative: {str(e)}", 'error')
    return render_template('create_cooperative.html')

@app.route('/cooperative/<name>')
def cooperative_details(name):
    coop = blockchain.get_cooperative(name)
    if coop:
        return render_template('cooperative_details.html', cooperative=coop)
    flash(f"Cooperative '{name}' not found.", 'error')
    return redirect(url_for('index'))

@app.route('/cooperative/<coop_name>/create_proposal', methods=['GET', 'POST'])
def create_proposal(coop_name):
    coop = blockchain.get_cooperative(coop_name)
    if not coop:
        flash(f"Cooperative '{coop_name}' not found.", 'error')
        return redirect(url_for('index'))

    if 'user_did' not in session:
        session['user_did'] = blockchain.create_did()

    if request.method == 'POST':
        creator = request.form['creator']
        description = request.form['description']
        proposal_type = ProposalType[request.form['proposal_type']]
        voting_period = int(request.form['voting_period'])
        voting_strategy = VotingStrategy[request.form['voting_strategy']]
        required_majority = float(request.form['required_majority'])

        if creator not in coop.members:
            flash("Creator must be a member of the cooperative.", 'error')
        elif creator != session['user_did'] and not coop.is_admin(session['user_did']):
            flash("You can only create proposals for yourself unless you're an admin.", 'error')
        else:
            proposal_id = coop.create_proposal(creator, description, proposal_type, voting_period, voting_strategy, required_majority)
            if proposal_id is not None:
                flash(f"Proposal created with ID: {proposal_id}", 'success')
                return redirect(url_for('cooperative_details', name=coop_name))
            else:
                flash("Failed to create proposal.", 'error')

    return render_template('create_proposal.html', cooperative=coop, proposal_types=ProposalType, voting_strategies=VotingStrategy, user_did=session['user_did'])

@app.route('/routes')
def list_routes():
    routes = []
    for rule in app.url_map.iter_rules():
        routes.append({
            "endpoint": rule.endpoint,
            "methods": ','.join(rule.methods),
            "route": str(rule)
        })
    return jsonify(routes)