# S01: WatchSession 반복 관측의 CPU 비용 분리

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 독립evidence crate 실행·원결과/통합·문서; metrics_recovery /tmp Rust 초안만
- 선행 작업: OS thread 이름 dlpad-library 비용실측·fresh scan0.328초

## 사용자 결과

전체scan과별개로100ms마다수행하는root metadata/identity 및native watcher비용을분리해최적화근거를얻는다.

## 계약과 범위

합성1만파일/단일owned process/private desktop CLI에서 control Sleep100ms×600과 실제WatchSession poll100ms×600을순차비교한다. dirty는기록만하고fullscan은실행하지않는진단이며제품정책을대체하지않는다. 프로세스/thread 실제CPU와전체10초원표본·실제elapsed·health/periodic관측·원fixture보존을기록한다. source/lock/dependency변경없음.

## 인수 조건

각구간원CPU·시간·dirty/health·bound·자연Job/private/input대조, missing/불확실원인구분. 가설: thread비용에서2회fresh scan을뺀나머지약0.25%가관측비용이면watch区간이control보다그만큼높다. 입증전주기관측을삭제하거나30초보완을변경하지않는다.

## 검증과 handoff

초안작성중이며현재제품은변경하지않았다. 전체build·GUI와동시측정하지않는다.

### 실제 첫 실행과 교정

원isolatedfixture lock생성은PASS, Clippy는연속 if 두표현의possible_missing_else 경고로exit101이었다. 원실패폴더를보존하고문장줄바꿈만교정한새evidence에서lock/clippy/build/physical구간을재실행한다. 제품source/CPU·workspace/lock 변경없음. raw90초는각phase상한이며실제60초×2라총180초진단예산이다. 공개WatchPoll의dirty/health로nativeevent와periodic원인별수를분리할수없어 unavailable로기록한다.

### 실제 비용 분리 종료

lock/clippy/release/physical4운영PASS와 source/현재CPU8f21불변·Job자연0/private/input을root/독립검토했다. control CPU0ticks(계측해상도아래),WatchSession process=thread평균0.4142025%·10초max0.776427%, dirty3/Active1/Unavailable0이었다. 원1만name/3B/sentinel·2원ID표본은보존했다. 공개API상native/periodic원인별수는미관측이고제품전체CPU로확대하지않는다. phase90초각설정과host210초는존재하나fixture/끝검증포함global180초절대deadline은없다. [원비용](../validation/evidence/windows-watch-poll-cost-fixed-20261006-continue/root-review.json).

다음가설은active poll마다mtime와identity의중복root metadata읽기이다. 같은원metadata로두필드를읽으면현재100ms/nativehint/30초보완정책변경없이비용감소가능하며별도수직작업으로red/비용/전체GUI를검증한다.
