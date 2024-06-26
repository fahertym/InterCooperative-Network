# icn/web/routes.py

from flask import render_template, request, redirect, url_for, flash, jsonify, session
from . import app
from ..blockchain import chain
from ..dao.governance import VotingStrategy, ProposalType, ProposalStatus

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
                # Add the current user as a member and admin
                if 'user_did' not in session:
                    session['user_did'] = blockchain.create_did()
                coop.add_member(session['user_did'], is_admin=True)
                flash(f"Cooperative '{name}' created successfully. You've been added as an admin member.", 'success')
                return redirect(url_for('cooperative_details', name=name))
            else:
                flash(f"Failed to create cooperative '{name}'.", 'error')
        except Exception as e:
            flash(f"Error creating cooperative: {str(e)}", 'error')
    return render_template('create_cooperative.html')

@app.route('/cooperative/<name>')
def cooperative_details(name):
    coop = blockchain.get_cooperative(name)
    if coop:
        return render_template('cooperative_details.html', cooperative=coop, ProposalStatus=ProposalStatus)
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

@app.route('/cooperative/<coop_name>/proposal/<int:proposal_id>/vote', methods=['POST'])
def vote_on_proposal(coop_name, proposal_id):
    coop = blockchain.get_cooperative(coop_name)
    if not coop:
        flash(f"Cooperative '{coop_name}' not found.", 'error')
        return redirect(url_for('index'))

    if 'user_did' not in session:
        flash("You must be logged in to vote.", 'error')
        return redirect(url_for('cooperative_details', name=coop_name))

    vote = request.form.get('vote') == 'yes'
    if coop.vote_on_proposal(proposal_id, session['user_did'], vote):
        flash("Vote cast successfully.", 'success')
    else:
        flash("Failed to cast vote.", 'error')

    return redirect(url_for('cooperative_details', name=coop_name))

@app.route('/cooperative/<coop_name>/proposal/<int:proposal_id>/execute', methods=['POST'])
def execute_proposal(coop_name, proposal_id):
    coop = blockchain.get_cooperative(coop_name)
    if not coop:
        flash(f"Cooperative '{coop_name}' not found.", 'error')
        return redirect(url_for('index'))

    if coop.execute_proposal(proposal_id):
        flash("Proposal executed successfully.", 'success')
    else:
        flash("Failed to execute proposal.", 'error')

    return redirect(url_for('cooperative_details', name=coop_name))

@app.route('/mine', methods=['POST'])
def mine():
    if 'user_did' not in session:
        flash("You must be logged in to mine.", 'error')
        return redirect(url_for('index'))

    try:
        if blockchain.mine_pending_transactions(session['user_did']):
            flash("Block mined successfully.", 'success')
        else:
            flash("Failed to mine block.", 'error')
    except ValueError as e:
        flash(str(e), 'error')

    return redirect(url_for('index'))

@app.route('/cooperative/<coop_name>/deploy_contract', methods=['GET', 'POST'])
def deploy_contract(coop_name):
    coop = blockchain.get_cooperative(coop_name)
    if not coop:
        flash(f"Cooperative '{coop_name}' not found.", 'error')
        return redirect(url_for('index'))

    if request.method == 'POST':
        code = request.form['code']
        contract_id = blockchain.deploy_contract(code)
        if contract_id:
            flash(f"Contract deployed successfully. Contract ID: {contract_id}", 'success')
            return redirect(url_for('cooperative_details', name=coop_name))
        else:
            flash("Failed to deploy contract.", 'error')

    return render_template('deploy_contract.html', cooperative=coop)

@app.route('/cooperative/<coop_name>/execute_contract/<contract_id>', methods=['POST'])
def execute_contract(coop_name, contract_id):
    coop = blockchain.get_cooperative(coop_name)
    if not coop:
        flash(f"Cooperative '{coop_name}' not found.", 'error')
        return redirect(url_for('index'))

    if blockchain.execute_contract(contract_id):
        flash("Contract executed successfully.", 'success')
    else:
        flash("Failed to execute contract.", 'error')

    return redirect(url_for('cooperative_details', name=coop_name))

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