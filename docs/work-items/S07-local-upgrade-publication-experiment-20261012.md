# S07: 로컬 버전 교체 publication 실험

- 상태: in-progress
- 분류: experiment
- 우선순위: P0
- 요구사항: R11 (ROADMAP S07 설치·업그레이드·rollback)
- 담당/수정 범위: root 통합·실행; diagnostic_cache는 /tmp 후보만 담당; 이 문서
- 선행 작업: 기존 설치 native identity·journal 대조

## 사용자 결과

향후 로컬 버전 업그레이드와 rollback에서 기존 설치를 보존하며 새 버전 선택을 공개하는 구현의 근거를 만든다. 실험 통과만으로 업그레이드 제품 완료를 주장하지 않는다.

## 계약과 범위

ROADMAP S07, 설치 ADR와 ARCHITECTURE의 소유권·취소 계약을 따른다. /tmp 후보 C# native 실험을 소유한 합성 폴더에서 실행해 원 parent/stage HANDLE 기반 create-only rename의 지원 여부·충돌·부분 파일 보존을 확인한다. 프로젝트 밖 기존 사용자 설치를 변경하지 않는다. 기존 파일 교체 fallback, 관리자 권한, 네트워크 updater는 범위 밖이다. Cargo/schema 변경 없음. 제안 아키텍처 채택은 실제 결과 뒤 기록한다.

## 인수 조건

실제 Windows에서 새 publication과 충돌 거절을 원 HANDLE ID·크기·hash와 함께 확인하고 old/new·충돌·partial 입력을 보존한다. 원 프로세스 자연 종료와 Job0 및 입력 데스크톱 보존을 확인한다. 미지원 API면 실패를 보존하고 경로 기반 덮어쓰기로 우회하지 않는다.

## 검증과 handoff

후보 collector 준비 중이며 native 실행은 root 담당이다. 원 standalone probe는 아직 실행 증거가 없고 실제 제품 launcher/반복upgrade/rollback 구현은 남아 있다.

- root 실제 compile/publication session15828: C# compile 성공, fresh publication 단계 native error87로 실패/exit1, collector terminal guard가 거절했다. 원 app/job/private desktop/input desktop 수명 로그 보존. 충돌·부분 파일 후속 단계는 실행하지 못했으며 전체 인수는 미통과다. absolute 경로 덮어쓰기 fallback 없음.
- 공식 FILE_RENAME_INFO 문서는 relative RootDirectory를 허용한다고 설명하지만 이 실제 실행은 성공하지 않았다. API/버퍼/지원 조건을 후속 실험으로 조사하고 proposed 아키텍처 채택은 보류한다. [공식 계약](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info).

- v2 actual50729 exit0은 진단 수집 완료이며 전체 API 성공이 아니다. Win32 win20/win24 모두 NTFS에서 native87 실패, 별도 NT nt20/nt24 모두 fresh rename·cold old/new·충돌 보존·partial4B·원 FileID128/원 root ID 보존 통과. 원 source frozen before/after와 원 pins/raw/Job0를 root가 독립 대조해 root-scoped-review.json 보존. 지금 Rust source는 후속 UI 수정으로 달라 역사 snapshot 증거로만 인수한다.
- 추가4바이트 buffer만으로 Win32 실패가 해결되지 않았고, 원 parent HANDLE-relative NT API를 다음 제품 adapter의 실험 후보로 선택한다. 제품 FreshRoot 연결/lease·revision chain·반복 upgrade/rollback과 power-loss는 미구현·미검증이다. Win32 실패를 NT 성공으로 소급하지 않는다.
