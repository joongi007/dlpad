# S01: 실제 Windows worker 스레드별 CPU 귀속

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: root 본문·probe 게시/native실행·통합 검증; metrics_recovery /tmp C# thread identity 초안만
- 선행 작업: 최신0478CPU 전체검증 및격리GUI3회 평균1.6%/strict실패

## 사용자 결과

목록/지표/UI 비용을 추측 대신 원 스레드 identity와 OS description으로 구분한다.

## 계약과 범위

기존private desktop/owned Job/10초×6·60초×3·80MiB 예산을 유지한다. 원app TID+creationFILETIME·CPU·GetThreadDescription을128thread이내수집하고 원QueryLimited HANDLE/반환문자열을해제한다. 이름없음/권한실패/재사용/threads변경은미확정으로기록하고관측되지않은guardian을포함했다고주장하지않는다. Rust/Cargo/lock/schema 변경없음. 프로파일 시험만으로 최신defaultCPU 예산통과를선언하지않는다.

## 인수 조건

원DLL/해제·원appthread membership/creationID 대조, 원측정구간·wholeJobCPU와thread합계의차이보존, 코드의명시Builder이름과실제OS이름일치, cap/실패경로기록·private/input/Job정상종료. 이후원인별구현과defaultfull/GUI 재검증을통해완료한다.

## 검증과 handoff

이전3f9GUI 원main CPU는0.10~0.13%였으며 나머지스레드역할은미확정이었다. observer는원ownedJob밖이고당시guardian은관측되지않았다. 최신메타데이터보정의실제scan0.328초/GUI1.6%실패를보존하고귀속초안을진행한다.

### 최신 원결과 검토

실제3회 평균CPU1.89925/1.61249/1.63821%·10초max4.54340/4.36435/4.22075%로 전부FAIL이다. 원OS description dlpad-library threadCPU1.63908/1.35241/1.37817%가 가장 크다. thread churn 때문에 전체thread·작업단계 귀속은 미확정이다. root는 원3만bytes·readonlyDB1만행/무결성·Job자연0/private/input 유지와 독립18구간 계산을 확인했다. 당시 source before==after, 이후 library.rs fixture 추가는 별도snapshot으로 구분한다. [원결과](../validation/evidence/windows-thread-description-60sec-20261005-continue/root-review.json). 이수집으로 CPU 인수를 완료하지 않는다.
