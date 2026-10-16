# S04: 자막 선택 native GUI 격리 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: diagnostic_cache agent; scripts/subtitle_ui_probe.py, 이 계약, docs/validation/subtitle-ui-validation.md와 전용 evidence만
- 선행 작업: 자막 native UI 및 주 담당의 새 release binary 신호

## 사용자 결과

저장 안 함/한국어/영어/둘 다와 auto 선택이 실제 GUI에서 typed argv로 전달되고 구간 충돌과 실행 중 선택 freeze가 지켜지는지 확인한다.

## 계약과 범위

README→PRODUCT→CONTEXT→관련 R04/R05/R08→ADR0002/0003 및 ARCHITECTURE/INTEGRATIONS를 읽었다. Xvfb만 사용하며 visible fallback 금지. 임시 sibling app-data/media와 합성 Python yt-dlp executable만 사용한다. 실제 영상/사이트/계정은 사용하지 않는다. 기존 probes와 app/engine/Cargo 수정 또는 build 금지. 주 담당이 binary 준비를 알리기 전 실제 GUI 실행하지 않는다.

## 인수 조건

선택별 실제 argv와 DB options/snapshot, preview screenshot, 구간 충돌 시 nojob/noexecute, running 중 다른 선택 클릭 후 원래 snapshot/선택 보존, 소유 프로세스 정리와 explicit binary SHA 기록. 인자/fixture 검증과 GUI 결과를 구분한다.

## 검증과 handoff

주 담당이 제공한 새 binary SHA256 `7e3afa82764e4bc52b254f337db39c81431efede22c7dce9b3bb2aa8aafdfa3d`로 실제6case/ko-auto-off 통과. 저장 안 함/ko/en/both가 각각1/2/2/3합성 artifacts로 Downloaded, 구간 충돌은 nojob/noexecute, running disabled 선택 클릭은 원래 snapshot/options와 후속 두번째 GUI요청의 both+auto를 보존했다. Preview/오류/running 화면을 직접 검토했다. conflict 오류 문구 OCR은 없어 JSON native_gui_verified=false를 유지한다.

AST+인자10건+합성 executable version/metadata/언어 framing5회 통과. 실제 명령/시간과7JSON/9PNG/layout evidence는 [전용 검증](../validation/subtitle-ui-validation.md)에 기록했다. 기존probe/app/engine/Cargo/build 변경 없음. other OS/실제 사이트 자막/IME/screen reader 미검증이며 주 담당 독립 검토를 요청했다.
