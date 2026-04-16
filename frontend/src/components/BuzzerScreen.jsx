import { useState, useEffect } from 'react';

export default function BuzzerScreen({ gameState, lastMessage, onBuzz, onLeave, playerName }) {
  const [buzzResult, setBuzzResult] = useState(null);
  const [buzzing, setBuzzing] = useState(false);

  const phase = gameState?.phase;
  const isPlayPhase = phase === 'play';
  const controllingTeamId = gameState?.controlling_team_id;
  const controllingTeam = gameState?.teams?.find((team) => team.id === controllingTeamId);
  const hasController = !!controllingTeam;
  const question = gameState?.current_question;
  const isGameOver = phase === 'game_over';
  const winnerScore = Math.max(0, ...(gameState?.teams || []).map((team) => team.score));
  const winners = (gameState?.teams || []).filter((team) => team.score === winnerScore && winnerScore > 0);

  useEffect(() => {
    if (lastMessage?.type === 'buzz_result') {
      setBuzzResult(lastMessage);
      setBuzzing(false);
    }
    if (lastMessage?.type === 'state' && lastMessage.phase === 'play' && !lastMessage.controlling_team_id) {
      setBuzzResult(null);
      setBuzzing(false);
    }
  }, [lastMessage]);

  const handleBuzz = () => {
    setBuzzing(true);
    onBuzz();
  };

  return (
    <div className="buzzer-screen">
      <div className="buzzer-header">
        <span className="player-name">{playerName}</span>
        <button className="btn btn-small btn-danger" onClick={onLeave}>Leave</button>
      </div>

      <div className="round-status">
        {isGameOver && (
          <div className="game-over-scene">
            <p className="winner-label">Game Over</p>
            <p className="winner-name">{winners.length > 1 ? 'It\'s a Tie!' : 'Winner'}</p>
            <p className="status-text active">
              {winners.length > 1
                ? winners.map((team) => team.name).join(' & ')
                : (winners[0]?.name || 'No winner')}
            </p>
            <p className="status-text waiting">Final score: {winnerScore} pts</p>
          </div>
        )}

        {!isGameOver && (
          <div className="player-question-board">
            <h3>{question?.text || 'Waiting for question...'}</h3>
            <ul className="player-answer-list">
              {(question?.answers || []).map((answer, index) => (
                <li key={index} className={`player-answer-item ${answer.revealed ? 'revealed' : ''}`}>
                  <span className="player-answer-index">{index + 1}</span>
                  <span className="player-answer-text">{answer.revealed ? answer.text : '••••••••'}</span>
                  <span className="player-answer-points">{answer.revealed ? answer.points : '?'}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {!isGameOver && !isPlayPhase && !hasController && (
          <p className="status-text waiting">Waiting for play phase...</p>
        )}
        {!isGameOver && isPlayPhase && !hasController && (
          <p className="status-text active">ROUND IS LIVE!</p>
        )}
        {!isGameOver && hasController && (
          <div className="winner-display">
            <p className="winner-label">BUZZED FIRST</p>
            <p className="winner-name">{controllingTeam.name}</p>
          </div>
        )}
      </div>

      <button
        className={`buzz-button ${isPlayPhase && !hasController ? 'enabled' : 'disabled'} ${buzzing ? 'buzzing' : ''}`}
        onClick={handleBuzz}
        disabled={!isPlayPhase || hasController || buzzing || isGameOver}
      >
        {buzzing ? '...' : 'BUZZ!'}
      </button>

      {buzzResult && (
        <div className={`buzz-feedback ${buzzResult.success ? 'success' : 'fail'}`}>
          {buzzResult.success ? '🎉 You buzzed first!' : buzzResult.error}
        </div>
      )}
    </div>
  );
}
