import { useState } from 'react';
import PlayerList from './PlayerList';

export default function HostPanel({ gameState, lastMessage, onAction }) {
  const [customFilename, setCustomFilename] = useState('');

  const teams = gameState?.teams || [];
  const controllingTeamId = gameState?.controlling_team_id;
  const controllingTeam = teams.find((team) => team.id === controllingTeamId);
  const question = gameState?.current_question;
  const isGameOver = gameState?.phase === 'game_over';
  const totalQuestions = gameState?.total_questions ?? 0;
  const currentQuestionDisplay = totalQuestions > 0 ? (gameState?.current_question_index ?? 0) + 1 : 0;
  const winnerScore = Math.max(0, ...teams.map((team) => team.score));
  const winners = teams.filter((team) => team.score === winnerScore && winnerScore > 0);
  const winnerIdForList = isGameOver && winners.length === 1 ? winners[0].id : controllingTeamId;

  const actionError =
    lastMessage?.type === 'action_result' && !lastMessage.success ? lastMessage.error : null;

  const handleRemoveTeam = (teamId) => {
    onAction({ action: 'remove_team', team_id: teamId });
  };

  const handleAwardPoint = (answerIndex) => {
    if (!controllingTeamId) return;
    onAction({ action: 'award_point', team_id: controllingTeamId, answer_index: answerIndex });
  };

  const handleLoadQuestions = () => {
    if (customFilename.trim()) {
      onAction({ action: 'load_questions', filename: customFilename.trim() });
      setCustomFilename('');
    }
  };

  return (
    <div className="host-panel">
      <div className="host-header">
        <h2>Game Admin</h2>
      </div>

      <div className="host-controls">
        {isGameOver && (
          <div className="game-over-scene admin-game-over">
            <p className="winner-label">Game Over</p>
            <p className="winner-name">{winners.length > 1 ? 'Tie Game' : 'Champion Team'}</p>
            <p className="status-text active">
              {winners.length > 1
                ? winners.map((team) => team.name).join(' & ')
                : (winners[0]?.name || 'No winner')}
            </p>
            <p className="status-text waiting">Final score: {winnerScore} pts</p>
          </div>
        )}

        <div className="question-board">
          <div className="question-board-header">
            <span>Question {currentQuestionDisplay} / {totalQuestions}</span>
            <span className="question-board-phase">{gameState?.phase || 'loading'}</span>
          </div>
          <h3 className="question-board-title">{question?.text || 'No active question yet'}</h3>

          <ul className="board-answer-list">
            {(question?.answers || []).map((answer, index) => (
              <li key={index} className={`board-answer-item ${answer.revealed ? 'revealed' : ''}`}>
                <span className="board-answer-index">{index + 1}</span>
                <span className="board-answer-text">{answer.full_text || answer.text}</span>
                <span className="board-answer-points">{answer.revealed ? answer.points : '?'}</span>
              </li>
            ))}
          </ul>
        </div>

        <div className="round-info">
          <div className={`status-badge ${gameState?.phase === 'play' ? 'live' : 'idle'}`}>
            {gameState?.phase || 'loading'}
          </div>

          {controllingTeam && (
            <div className="winner-card">
              <span className="winner-card-label">Controlling</span>
              <span className="winner-card-name">{controllingTeam.name}</span>
            </div>
          )}
        </div>

        <div className="question-loader">
          <h3>Load Question Set</h3>
          <div className="loader-input">
            <input
              type="text"
              placeholder="e.g., questions.json or easy_questions.json"
              value={customFilename}
              onChange={(e) => setCustomFilename(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleLoadQuestions()}
            />
            <button className="btn btn-secondary" onClick={handleLoadQuestions} disabled={!customFilename.trim()}>
              Load Questions
            </button>
          </div>
        </div>

        <div className="admin-grid">
          <button className="btn btn-primary" onClick={() => onAction({ action: 'start_game' })}>
            Start Game
          </button>
          <button className="btn btn-primary" onClick={() => onAction({ action: 'start_round' })} disabled={isGameOver}>
            Start Round
          </button>
          <button className="btn btn-secondary" onClick={() => onAction({ action: 'strike' })} disabled={isGameOver}>
            Strike
          </button>
          <button className="btn btn-secondary" onClick={() => onAction({ action: 'next_question' })} disabled={isGameOver}>
            Next Question
          </button>
          <button className="btn btn-danger" onClick={() => onAction({ action: 'reset_game' })}>
            Reset Game
          </button>
        </div>

        {actionError && <p className="admin-error">{actionError}</p>}

        <div className="question-panel">
          <h3>Judge Answers</h3>
          <p>Select revealed answers to award points to controlling team.</p>

          {question?.answers?.length > 0 && (
            <ul className="answer-list">
              {question.answers.map((answer, index) => (
                <li key={index} className="answer-item">
                  <span>
                    {answer.revealed ? (answer.full_text || answer.text) : `Answer ${index + 1}`} ({answer.points})
                  </span>
                  <button
                    className="btn btn-small btn-secondary"
                    onClick={() => handleAwardPoint(index)}
                    disabled={answer.revealed || !controllingTeamId || isGameOver}
                  >
                    Award
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      <PlayerList players={teams} winnerId={winnerIdForList} onRemoveTeam={handleRemoveTeam} />

      <p className="admin-footer">Round points: {gameState?.round_points ?? 0} • Strikes: {gameState?.strikes ?? 0}</p>
    </div>
  );
}
