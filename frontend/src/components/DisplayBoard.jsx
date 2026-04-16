export default function DisplayBoard({ gameState }) {
    const teams = [...(gameState?.teams || [])].sort((a, b) => b.score - a.score);
    const question = gameState?.current_question;
    const phase = gameState?.phase || 'lobby';
    const controllingTeam = teams.find((team) => team.id === gameState?.controlling_team_id);
    const winnerScore = Math.max(0, ...teams.map((team) => team.score));
    const winners = teams.filter((team) => team.score === winnerScore && winnerScore > 0);
    const totalQuestions = gameState?.total_questions ?? 0;
    const currentQuestionDisplay = totalQuestions > 0 ? (gameState?.current_question_index ?? 0) + 1 : 0;

    const phaseLabel = {
        lobby: 'Waiting for host',
        play: 'Buzz in now',
        answer: controllingTeam ? `${controllingTeam.name} is answering` : 'Answer phase',
        round_over: 'Round complete',
        game_over: 'Game over',
    }[phase] || phase;

    return (
        <div className="display-board">
            <div className="display-header">
                <div>
                    <p className="display-kicker">Family Feud</p>
                    <h1>Live Game Board</h1>
                </div>
                <div className="display-phase-badge">{phaseLabel}</div>
            </div>

            {phase === 'game_over' && (
                <div className="display-champion">
                    <p className="winner-label">Champion</p>
                    <p className="winner-name">{winners.length > 1 ? 'Tie Game' : (winners[0]?.name || 'No winner')}</p>
                    <p className="status-text waiting">{winners.length > 1 ? winners.map((team) => team.name).join(' • ') : `Final score: ${winnerScore} pts`}</p>
                </div>
            )}

            <div className="display-layout">
                <section className="display-main-panel">
                    <div className="display-question-meta">
                        <span>Question {currentQuestionDisplay} / {totalQuestions}</span>
                        <span>Round points: {gameState?.round_points ?? 0}</span>
                    </div>

                    <h2 className="display-question-title">{question?.text || 'Waiting for the next question...'}</h2>

                    <ul className="display-answer-grid">
                        {(question?.answers || []).map((answer, index) => (
                            <li key={index} className={`display-answer-card ${answer.revealed ? 'revealed' : ''}`}>
                                <span className="display-answer-index">{index + 1}</span>
                                <span className="display-answer-text">{answer.revealed ? answer.text : '••••••••••••'}</span>
                                <span className="display-answer-points">{answer.revealed ? answer.points : '?'}</span>
                            </li>
                        ))}
                    </ul>
                </section>

                <aside className="display-sidebar">
                    <div className="display-side-card">
                        <h3>Teams</h3>
                        <ul className="display-team-list">
                            {teams.map((team) => (
                                <li
                                    key={team.id}
                                    className={`display-team-item ${team.id === gameState?.controlling_team_id ? 'controlling' : ''}`}
                                >
                                    <span className="display-team-rank">#{team.id}</span>
                                    <span className="display-team-name">{team.name}</span>
                                    <span className="display-team-score">{team.score}</span>
                                </li>
                            ))}
                        </ul>
                    </div>

                    <div className="display-side-card">
                        <h3>Round Status</h3>
                        <p className="display-status-line">
                            {controllingTeam ? `Control: ${controllingTeam.name}` : 'No team in control yet'}
                        </p>
                        <p className="display-status-line">Strikes: {'✕ '.repeat(gameState?.strikes || 0) || 'None'}</p>
                    </div>
                </aside>
            </div>
        </div>
    );
}
