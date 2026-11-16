(define (thunk f)
    (vector 'thunk f))

(define (force x)
    (if (and (vector? x) (eq? (vector-ref x 0) 'thunk))
        (let ((v (force ((vector-ref x 1)))))
            (vector-set! x 0 'value)
            (vector-set! x 1 v)
            v)
    (if (and (vector? x) (eq? (vector-ref x 0) 'value))
        (vector-ref x 1)
    x)))

(define (apply-lazy f arg)
    ((force f) arg))

(define (make-cons tag fields)
    (vector 'cons tag fields))

(define (cons-is? tag x)
    (eq? (vector-ref x 1) tag))

(define (cons-field x n)
    (vector-ref (vector-ref x 2) n))

; primitive functions

(define (trace msg x)
    (display (force msg)) x)

(define (io-pure x)
    (lambda () x))

(define (io-bind m f)
    (lambda ()
        ((force (apply-lazy f ((force m)))))))

(define (int-add x y) (+ (force x) (force y)))
(define (int-sub x y) (- (force x) (force y)))
(define (int-mul x y) (* (force x) (force y)))
(define (int-div x y) (/ (force x) (force y)))
(define (int-gt x y) (if (> (force x) (force y)) 1 0))
(define (int-lt x y) (if (< (force x) (force y)) 1 0))

(define (str-int x) (number->string (force x)))
(define (str-len x) (string-length (force x)))
(define (str-concat x y) (string-append (force x) (force y)))
(define (str-char n s)
    (if (< (force n) (string-length (force s)))
        (string (string-ref (force s) (force n)))
        ""))

(define (str-take n s)
    (substring (force s) 0 (force n)))

(define (str-skip n s)
    (substring (force s) (force n) (string-length (force s))))

(define (str-cmp a b)
    (if (string<? (force a) (force b))
        -1
    (if (string>? (force a) (force b))
        1
        0)))

(define (str-find sub str)
  (let ((sub (force sub)))
  (let ((str (force str)))
  (let ((n (string-length str))
        (m (string-length sub)))
    (let loop ((i 0))
      (cond
        ((> (+ i m) n) -1)
        ((string=? (substring str i (+ i m)) sub) i)
        (else (loop (+ i 1)))))))))

; external functions

(define externs
    (make-hashtable string-hash string=?))

(define (extern name)
    (hashtable-ref externs name name))

(hashtable-set! externs "io::print"
    (lambda (x)
        (lambda ()
            (display (force x))
            (void))))

(hashtable-set! externs "fs::isFile"
    (lambda (path)
        (lambda ()
            (if (and (file-exists? (force path)) (not (file-directory? (force path))))
                1 0))))

(hashtable-set! externs "fs::readFile"
    (lambda (path)
        (lambda ()
            (call-with-input-file (force path)
                (lambda (port)
                    (get-string-all port))))))

; generated code
