; ===== SPECIAL COMMANDS =====
(define saved-path '())

; Function for reversing a direction.
(define reverse-dir
  (lambda (dir)
    (cond ((string=? dir "n") "s")
      ((string=? dir "s") "n")
      ((string=? dir "e") "w")
      ((string=? dir "w") "e")
      (else #f))))

; Function for handling special commands.
(define run-command
  (lambda (cmd)
    (cond
      ((string=? cmd "reload") ; Reload the config file.
       (list (tome:reload-config)
             (tome:write-scrollback "Reloaded the config file.\n")))
      ((string=? cmd "path") (set! saved-path '()) '()) ; Start a new path.
      ((string=? cmd "addpath") (set! saved-path (cons "n" saved-path)) ; Start a new path.
                                (tome:write-scrollback "added\n"))
      ((string=? cmd "backtrack") ; Backtrack to the path start.
       (let ((backpath saved-path))
         (set! saved-path '())
         (append (map tome:send (map reverse-dir backpath))
                 (list (tome:write-scrollback "backtracking")))))
      (else (list (tome:write-scrollback
                    (string-append (string-append "Invalid command: " cmd) "\n")))))))

; ===== ALIASES =====
(define aliases (make-hash-table))

; Function for defining an alias.
(define define-alias
  (lambda (alias command)
    (hash-set! aliases alias command)))

; ===== COMMAND SENDING ======
; Function to run on input. Returns a list of actions to perform.
(define send-input-hook
  (lambda (input)
    (cond
      ((string=? input "") (list (tome:send input))) ; Empty input.
      ((string-contains input ";")
       (apply append (map send-input-hook (string-split input #\;)))) ; Multiple commands.
      ((hash-ref aliases input)
       (send-input-hook (hash-ref aliases input))) ; Aliases (recursive).
      ((string-prefix? "#" input) ; Command
       (run-command (substring input 1)))
      ;((string-prefix? "/" input) ; Search.
      ; (begin
      ;   (search-backwards (substring input 1))
      ;   '())) ; Note that an empty list is returned.
      (else (list (tome:send input)))))) ; Everything else.

; ===== MUD-SPECIFIC STUFF =====
(define-alias "test" "4n4e")
