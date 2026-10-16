# S01: WatchSession root 관측의 중복 metadata 조회 제거

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/watch.rs·CONTEXT/ADR0002·원실행/통합; metrics_recovery 읽기검토/tmp 독립비용감사
- 선행 작업: scan없는관측평균0.4142%실증/현재8f21페이지snapshot

## 사용자 결과

동일100ms폴더감시정책에서불필요한root metadata재조회를줄인다.

## 계약과 범위

mtime와root identity를단일metadata관측에서얻고100ms호출/nativehint처리·ordinary1초gate·backend실패즉시flush·reattach2초/30초보완을유지한다. 메타데이터관측은snapshot이고OSnamespace를고정하는권한이아니다. 기존root종류/symlink/error/reconnect 검사를보존한다. 의존성/schema/lock변경없음. 현재개발ZIP2입력sourcefreeze가끝나기전Rust수정없음.

## 인수 조건

active poll의원wrapper조회2→1 red/green·mtime/error/identityloss/reattach/periodic 회귀유지. 원control/poll60초 비용·원fixture/source/CPUguard를재비교하고최신전체/GUI예산은실제검증한다. 이조회제거만으로P0CPU성공을미리주장하지않는다.

## 검증과 handoff

원비용을근거로선택한다. 가설순서: 중복root조회, 단일Windows metadata조회비용, nativebackend고정비용. 각가설은동일probe에서조회하나의변경으로분리한다.

### 회귀 계측 시작

패키지생성/폰트/원allentry리뷰종료후root만watch.rs를수정했다. cfgtest thread-local 계측은WatchSession root_metadata wrapper조회만측정하며Watcher::new의별도metadata와kernel내부호출수를포함했다고주장하지않는다. oldpoll의mtime/identity2조회에서1조회조건red를nativeWindows로실행한다. 기존2s/30s/ordinary·health 계약은동결한다.

### 실제 red/green

Windows release active_poll 단일관측 회귀가 기존2조회 vs 기대1로 실패(exit101)했다. tuple 관측을 도입해 active identity/mtime를 공유하고 attach 전후 identity 검사 및 aftermtime 재사용을 유지했다. Windows watch::tests 전체는 exit0 통과했다. 원 로그/실패 결과를 보존했다. 동일60초 control/60초 watcher 비용 재시험 진행 중이며 성능 성공은 아직 주장하지 않는다.

### 비용 전후와 통합 검증

동일조건control60초/watch600회60초의한쌍에서평균0.4142%→0.12954%,새10초최대0.31085%였다. control은CPU계측해상도아래였다. root가1만정확파일명/3byte전체와sentinel보존을직접확인했다. 이것은scan없는감시비용이며제품GUI인수는별도다. 첫전체fmt는줄바꿈1건으로exit1실패했으며원결과를보존했다. 줄바꿈교정후새경로전체검증진행중이다.

### 최신 전체 검증

windows-watch-single-workspace-green-20261006-continue의fmt/clippy/test/release4개exit0과sourcebaseline/final의현재전체일치를root가확인했다. 새CPUce7efd9732788f6136e84d4ecc90bd00bc7708381e691d498bf4726eef8a1012/7895040B다. 제품격리GUI60초×3은별도실행중이며이작업성능인수는아직완료하지않았다.

### 제품 GUI 결과와 완료 범위

최신CPUce7efd의격리GUI60초×3은평균1.6915/1.4570/1.9770%,10초max4.5413/4.2074/4.8305%로CPU FAIL이며메모리는PASS다. 원3만payload·sentinel·readonlyDB각1만행/integrity를root가검증했다. 중복조회제거/실패회귀/통합/비용실측작업은완료했으나P0성능게이트는미완료다. 실제worker대기및반복scan비용분리를후속으로진행한다.
