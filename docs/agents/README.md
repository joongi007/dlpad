# 에이전트 독립 작업 환경

진입점은 루트 AGENTS.md 하나다. CLAUDE.md는 해당 문서로 연결한다. 다른 에이전트는 시작 프롬프트에 `AGENTS.md를 읽고 작업 <ID>를 수행하라`를 포함한다. 규칙을 서로 다른 파일에 복사해 유지하지 않는다.

현재 tracker: local markdown, 위치 `docs/work-items/`. 상태: proposed → ready → in-progress → review → done, 보조 blocked. 분류: bug, feature, docs, maintenance. 우선순위: P0/P1/P2. 외부 이슈 생성·알림 전송은 이 환경에 포함하지 않는다.

작업 문서는 [템플릿](WORK_ITEM_TEMPLATE.md)을 복사한다. 다음 담당자는 요구사항 ID, 최근 결정, 파일 소유, 재현 명령, 검증 결과, 남은 문제를 읽으면 이어갈 수 있어야 한다. 비밀·브라우저 쿠키·개인 절대 경로를 handoff에 쓰지 않는다.

사람 또는 에이전트 역할은 구현·검토·통합으로 구분하되 모델 이름에 역할을 고정하지 않는다. 병렬 작업이 필요하면 coordinator가 먼저 파일 범위와 public interface를 나누고 통합 검사 책임자를 지정한다.
